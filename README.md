# Tokio Async Tutorial Repo \ Mini-Redis

[Tokio Async Tutorial Website](https://tokio.rs/tokio/tutorial)
# run notes
uses: pre-fab **mini-redis** to interact with (at least initially)
`cargo install mini-redis` ~~> `mini-redis-server`, `mini-redis-client`

# Personal Notes

- Parallelism, Concurrency, and Reorderability are all apiece as resource allotment options in the face of *logical-nondependency*
```
standard:       AAA  BBB  CCC
reorder:        CCC  AAA  BBB
paralalelism:   AAA
                BBB
                CCC
concurrency:    AA B C BB C A C
```
        - reordering is little talked about, though certainly the compiler and cpu both do it to varying degrees
                - unclear how much benefit might be accrued if all options for it were noted and utilized  (could be quite small, or there may be notable efficiencies)
                        - an aside: is there something like compression, but for time: where we abstract representations of processes in order to reduce redundancy (e.g. two bits of operation that are the same and thus only need to be done once?)
        - parallelism & concurrency, as used, have syntax that's heavily involved in the sorts of processes they are used for and hardware characteristics that leverage them
                - e.g. we don't get "concurrency channels" typically because the delay(ing) processes are actually parallel processors that we have no control over
        - a core part of the syntax of both (and all three) however is separating necessarilly serialized from nondependent computations
