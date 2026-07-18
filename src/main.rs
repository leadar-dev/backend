mod broker;
mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod services;

use std::sync::Arc;

use anyhow::Context;
use axum::{middleware as axum_middleware, routing::{get, patch, post}, Router};
use axum_prometheus::PrometheusMetricLayer;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing::info;

use broker::publisher::Publisher;
use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub publisher: Arc<Publisher>,
    pub config: Arc<Config>,
}

fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_env("LOGGING__LEVEL")
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(fmt::layer().json())
        .init();
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cfg = Config::from_env().context("invalid config")?;
    info!(port = cfg.server.port, "config loaded");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database.url)
        .await
        .context("database connection failed")?;
    info!("database connected");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("migrations failed")?;
    info!("migrations applied");

    let publisher = Arc::new(
        Publisher::new(&cfg.broker.url)
            .await
            .context("publisher init failed")?,
    );

    let cfg = Arc::new(cfg);

    let state = AppState {
        pool: pool.clone(),
        publisher: Arc::clone(&publisher),
        config: Arc::clone(&cfg),
    };

    // Start main consumer in background with auto-restart
    {
        let pool_clone = pool.clone();
        let publisher_clone = Arc::clone(&publisher);
        let broker_url = cfg.broker.url.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = broker::consumer::start_consumer(
                    &broker_url,
                    pool_clone.clone(),
                    Arc::clone(&publisher_clone),
                )
                .await
                {
                    tracing::error!(err = %e, "consumer crashed, restarting in 5s");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    }

    // Start categories consumer in background with auto-restart
    {
        let pool_clone = pool.clone();
        let broker_url = cfg.broker.url.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = broker::categories_consumer::start_categories_consumer(
                    &broker_url,
                    pool_clone.clone(),
                )
                .await
                {
                    tracing::error!(err = %e, "categories consumer crashed, restarting in 5s");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    }

    // Start DLQ consumer in background with auto-restart
    {
        let broker_url = cfg.broker.url.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = broker::dlq::start_dlq_consumer(&broker_url).await {
                    tracing::error!(err = %e, "DLQ consumer crashed, restarting in 5s");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    }

    // Start z-score scheduler: recalculates every 30 minutes
    {
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = services::analytics::calculate_zscores(&pool_clone).await {
                    tracing::error!(err = %e, "z-score calculation failed");
                }
                tokio::time::sleep(tokio::time::Duration::from_mins(30)).await;
            }
        });
    }

    let (prometheus_layer, metrics_handle) = PrometheusMetricLayer::pair();

    let jwt_secret = cfg.auth.jwt_secret.clone();
    let jwt_secret_admin = jwt_secret.clone();

    let protected_routes = Router::new()
        .route("/wants", get(handlers::wants::get_wants))
        .route("/wants/:id", get(handlers::wants::get_want_by_id))
        .route("/categories", get(handlers::categories::get_categories))
        .route("/analytics/zscore", get(handlers::analytics::get_zscore))
        .route("/analytics/heatmap", get(handlers::analytics::get_heatmap))
        .route("/users/me", get(handlers::users::get_users_me))
        .layer(axum_middleware::from_fn({
            let pool = pool.clone();
            move |jar, req, next| {
                let secret = jwt_secret.clone();
                let pool = pool.clone();
                crate::middleware::auth::require_auth_with_secret(jar, secret, pool, req, next)
            }
        }));

    let admin_routes = Router::new()
        .route("/admin/users", get(handlers::admin::get_admin_users))
        .route(
            "/admin/users/:telegram_id/access",
            patch(handlers::admin::patch_admin_user_access),
        )
        .route(
            "/admin/feature-flags",
            get(handlers::admin::get_admin_feature_flags),
        )
        .route(
            "/admin/feature-flags/:name",
            patch(handlers::admin::patch_admin_feature_flag),
        )
        .layer(axum_middleware::from_fn(middleware::role::require_admin))
        .layer(axum_middleware::from_fn({
            let pool = pool.clone();
            let jwt_secret_admin = jwt_secret_admin;
            move |jar, req, next| {
                let secret = jwt_secret_admin.clone();
                let pool = pool.clone();
                crate::middleware::auth::require_auth_with_secret(jar, secret, pool, req, next)
            }
        }));

    let app = Router::new()
        .route("/health", get(handlers::health::health))
        .route(
            "/metrics",
            get(move || async move { metrics_handle.render() }),
        )
        .route("/auth/telegram", post(handlers::auth::post_auth_telegram))
        .route("/auth/logout", post(handlers::auth::post_auth_logout))
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("failed to bind")?;
    info!(addr = %addr, "server started");

    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}
