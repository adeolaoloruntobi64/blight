use std::marker::PhantomData;

pub struct StaticClientConfig<const CAP: usize, P, S, Uint> {
    pub channel_size: usize,
    pub path: P,
    pub fallback_service: Option<S>,
    pub phantomdata: PhantomData<[Uint; CAP]>
}