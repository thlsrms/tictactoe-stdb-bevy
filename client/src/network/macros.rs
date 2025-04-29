/// Macro to register callbacks and subscriptions
macro_rules! stdb_subscribe {
    // Match update/insert/delete operation handlers
    ($app:ident, $conn:expr, $op:ident, $table:ident) => {
        paste::paste! {
            let (tx, rx) = crossbeam_channel::unbounded::<[<On $op:camel>]<$table>>();
            stdb_register_event!($app, [<On $op:camel>]<$table>, rx);
            _subscribe_op_impl!($conn, $op, $table, tx);
        }
    };

    // Match subcriptions
    ($app:ident, $conn:expr, $sub_query:expr, $evt_ty:ty, |$param:pat_param| $body:expr) => {
        stdb_subscribe!(
            $app,
            $conn,
            $sub_query,
            $evt_ty,
            move |$param: &SubscriptionEventContext| $body,
            false
        );
    };
    ($app:ident, $conn:expr, $sub_query:expr, $evt_ty:ty, |$param:pat_param| $body:expr, $sub_to_error:tt) => {
        stdb_subscribe!(
            $app,
            $conn,
            $sub_query,
            $evt_ty,
            move |$param: &SubscriptionEventContext| $body,
            $sub_to_error
        );
    };
    ($app:ident, $conn:expr, $sub_query:expr, $evt_ty:ty, $wrap:expr, $sub_to_error:tt) => {{
        let (tx, rx) = crossbeam_channel::unbounded::<OnSubApplied<$evt_ty>>();
        stdb_register_event!($app, OnSubApplied<$evt_ty>, rx);
        let sub_builder = $conn.subscription_builder().on_applied(move |ctx| {
            tx.send(OnSubApplied($wrap(ctx))).unwrap();
        });

        let sub_builder = {
            if $sub_to_error {
                let (tx_err, rx_err) = crossbeam_channel::unbounded::<OnSubError<$evt_ty>>();
                stdb_register_event!($app, OnSubError<$evt_ty>, rx_err);
                sub_builder.on_error(move |_, err| {
                    tx_err.send(OnSubError::new(err)).unwrap();
                })
            } else {
                sub_builder
            }
        };

        sub_builder.subscribe($sub_query);
    }};
}

/// Register the events for the lifecycle callbacks: OnConnect, OnConnectError, OnDisconnect
macro_rules! stdb_lifecycle_events {
    ($app:ident, $conn_builder:ident) => {{
        let (conn_tx, conn_rx) = crossbeam_channel::unbounded::<OnConnect>();
        let (conn_err_tx, conn_err_rx) = crossbeam_channel::unbounded::<OnConnectError>();
        let (disconn_tx, disconn_rx) = crossbeam_channel::unbounded::<OnDisconnect>();

        stdb_register_event!(
            $app,
            (OnConnect, conn_rx),
            (OnConnectError, conn_err_rx),
            (OnDisconnect, disconn_rx)
        );

        $conn_builder
            .on_connect(move |_ctx, identity, token: &str| {
                conn_tx.send(<OnConnect>::new(identity, token)).unwrap();
            })
            .on_connect_error(move |_ctx, err| {
                conn_err_tx.send(<OnConnectError>::new(err)).unwrap();
            })
            .on_disconnect(move |_ctx, err| {
                disconn_tx.send(<OnDisconnect>::new(err)).unwrap();
            })
    }};
}

// Register the Callbacks wrapped by a Stdb
//
// On the web we poll the connection so we can't subscribe before the it's polled,
// but we have to register the events before that.
macro_rules! stdb_register_event {
    ($app:ident, $evt_ty:ty, $rx:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        ($app).add_event::<Stdb<$evt_ty>>();
        #[cfg(not(target_arch = "wasm32"))]
        ($app).add_systems(bevy::prelude::PreUpdate, process_network_queue::<$evt_ty>);
        ($app).insert_resource(EventQueue::<$evt_ty>(Mutex::new($rx)));
    };

    ($app:ident, $( ($evt_ty:ty , $rx:ident) ),* ) => {
        $(
            stdb_register_event!($app, $evt_ty);
            ($app).insert_resource(EventQueue::<$evt_ty>(Mutex::new($rx)));
        )*
    };

    ($app:ident, $( $evt_ty:ty ),* ) => {
        $(
            ($app).add_event::<Stdb<$evt_ty>>();
            ($app).add_systems(bevy::prelude::PreUpdate, process_network_queue::<$evt_ty>);
        )*
    };
}

/// Helper macro that actually calls the right on_insert/on_delete/on_update
macro_rules! _subscribe_op_impl {
    // update needs (old, new) and a struct payload
    ($conn:expr, update, $table:ident, $tx:ident) => {
        paste::paste! {
            $conn
                .db.[<$table:snake>]()
                .on_update(move |_ctx, old, new| {
                    $tx
                        .send(OnUpdate { old: old.clone(), new: new.clone() })
                        .unwrap();
                });
        }
    };

    // insert or delete need a single `row` and a tuple‐like payload
    ($conn:expr, $op:ident, $table:ident, $tx:ident) => {
        paste::paste! {
            $conn
                .db.[<$table:snake>]()
                .[<on_ $op>](move |_ctx, row: &$table| {
                    $tx
                        .send([<On $op:camel>](row.clone()))
                        .unwrap();
                });
        }
    };
}
