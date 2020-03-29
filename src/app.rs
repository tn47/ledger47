struct Application<D>
where
    D: Store,
{
    backend: D,
}
