pub trait RadiusPipelineMutability {
    type Ref<'t, T>
    where
        T: 't;

    fn reborrow<'t1, 't2, T>(t: &'t1 mut Self::Ref<'t2, T>) -> Self::Ref<'t1, T>
    where
        't2: 't1,
        T: 't2;
}

pub struct SharedRadiusPipeline {}
pub struct MutatingRadiusPipeline {}

impl RadiusPipelineMutability for SharedRadiusPipeline {
    type Ref<'t, T>
        = &'t T
    where
        T: 't;

    fn reborrow<'t1, 't2, T>(t: &'t1 mut Self::Ref<'t2, T>) -> Self::Ref<'t1, T>
    where
        't2: 't1,
        T: 't2,
    {
        *t
    }
}

impl RadiusPipelineMutability for MutatingRadiusPipeline {
    type Ref<'t, T>
        = &'t mut T
    where
        T: 't;

    fn reborrow<'t1, 't2, T>(t: &'t1 mut Self::Ref<'t2, T>) -> Self::Ref<'t1, T>
    where
        't2: 't1,
        T: 't2,
    {
        &mut **t
    }
}
