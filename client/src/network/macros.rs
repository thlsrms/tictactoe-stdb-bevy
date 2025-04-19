/// Helper macro to register callbacks and subscriptions
macro_rules! stdb_event {
    // Match update handlers (ctx, old, new)
    ($app:ident, $conn:expr, $evt_ty:ident, $table:ident) => {
        let (tx, rx) = crossbeam_channel::unbounded::<$evt_ty<$table>>();
        paste::paste! {
            $conn.db.[<$table:snake>]().[<$evt_ty:snake>](move |_ctx, old, new| {
                tx.send(OnUpdate { old: old.clone(), new: new.clone() }).unwrap();
            });
        }
        register_network_event!($app, OnUpdate<$table>, rx);
    };

    // Match insert/delete/etc. handlers (ctx, row)
    ($app:ident, $conn:expr, $evt_ty:ident<$table:ty>) => {
        let (tx, rx) = crossbeam_channel::unbounded::<$evt_ty<$table>>();
        paste::paste! {
            $conn.db.[<$table:snake>]().[<$evt_ty:snake>](move |_ctx, row: &$table| {
                tx.send($evt_ty(row.clone())).unwrap();
            });
        }
        register_network_event!($app, $evt_ty<$table>, rx);
    };

    // Match subscription applied (1 argument: ctx)
    ($app:ident, $conn:expr, $sub_query:expr, $evt_ty:ty, $wrap:expr) => {{
        let (tx, rx) = crossbeam_channel::unbounded::<$evt_ty>();
        $conn
            .subscription_builder()
            .on_applied(move |ctx| {
                tx.send($wrap(ctx)).unwrap();
            })
            .subscribe($sub_query);
        register_network_event!($app, $evt_ty, rx);
    }};

    // Match subscription applied and error (1 argument: ctx)
    ($app:ident, $conn:expr, $sub_query:expr, $evt_ty:ty, $wrap:expr, $err_ty:ty, $wrap_err:expr) => {{
        let (tx, rx) = crossbeam_channel::unbounded::<$evt_ty>();
        let (tx_err, rx_err) = crossbeam_channel::unbounded::<$err_ty>();
        $conn
            .subscription_builder()
            .on_applied(move |ctx| {
                tx.clone().send($wrap(ctx)).unwrap();
            })
            .on_error(move |ctx, err| {
                tx_err.send($wrap_err(ctx, err)).unwrap();
            })
            .subscribe($sub_query);
        register_network_event!($app, $evt_ty, rx);
        register_network_event!($app, $err_ty, rx_err);
    }};
}

macro_rules! stdb_lifecycle {
    ($app:ident, $conn_builder:ident, ($conn_evt_ty:ty, $conn_err_evt_ty:ty, $disconn_evt_ty:ty)) => {{
        let (conn_tx, conn_rx) = crossbeam_channel::unbounded::<$conn_evt_ty>();
        register_network_event!($app, $conn_evt_ty, conn_rx);
        let (conn_err_tx, conn_err_rx) = crossbeam_channel::unbounded::<$conn_err_evt_ty>();
        register_network_event!($app, $conn_err_evt_ty, conn_err_rx);
        let (disconn_tx, disconn_rx) = crossbeam_channel::unbounded::<$disconn_evt_ty>();
        register_network_event!($app, $disconn_evt_ty, disconn_rx);
        $conn_builder
            .on_connect(move |_ctx, identity, token: &str| {
                conn_tx.send(<$conn_evt_ty>::new(identity, token)).unwrap();
            })
            .on_connect_error(move |_ctx, err| {
                conn_err_tx.send(<$conn_err_evt_ty>::new(err)).unwrap();
            })
            .on_disconnect(move |_ctx, err| {
                disconn_tx.send(<$disconn_evt_ty>::new(err)).unwrap();
            })
    }};
}

macro_rules! register_network_event {
    ($app:ident, $evt_ty:ty, $rx:ident) => {{
        if !($app).world().contains_resource::<EventQueue<$evt_ty>>() {
            ($app).insert_resource(EventQueue::<$evt_ty>(Mutex::new($rx)));
            ($app).add_event::<NetworkEvent<$evt_ty>>();
            ($app).add_systems(bevy::prelude::PreUpdate, process_network_queue::<$evt_ty>);
        }
    }};
}
