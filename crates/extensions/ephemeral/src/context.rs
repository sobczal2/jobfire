use jobfire_core::domain::job::context::ContextData;

pub struct EphemeralContext<TData: ContextData> {
    inner: TData,
}
