use axum::{Router, handler::Handler};

pub trait RouterExt {
    type S: Clone + Send + Sync + 'static;
    fn die_param<T1: 'static, T2: 'static>(
        self,
        param: &str,
        list_fn: impl Handler<T1, Self::S>,
        delete_fn: impl Handler<T2, Self::S>,
    ) -> Self;
}

impl<S> RouterExt for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    type S = S;
    fn die_param<T1: 'static, T2: 'static>(
        self,
        param: &str,
        list_fn: impl Handler<T1, S>,
        delete_fn: impl Handler<T2, S>,
    ) -> Self {
        self.route(
            &format!("/api/dies/{{die_id}}/{param}/{{id}}"),
            axum::routing::get(list_fn).delete(delete_fn),
        )
    }
}
