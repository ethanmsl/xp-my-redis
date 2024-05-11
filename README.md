# Tokio Async Tutorial Repo \ Mini-Redis

[Tokio Async Tutorial Website](https://tokio.rs/tokio/tutorial)
# run notes
uses: pre-fab **mini-redis** to interact with (at least initially)
`cargo install mini-redis` ~~> `mini-redis-server`, `mini-redis-client`


# Personal Notes


## Parallelism, Concurrency, and Reorderability are all apiece as resource allotment options in the face of *logical-nondependency*

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


## Concurrency shuffling only ocurs at `.await` points

So for some await breaks (|): 
```
AA|AAAA|A BBB|BBB
```

Allowably shuffles would *include*:
`AA BBB AAAA BBB A`
`AA BBB BBB AAAA A`
`AA AAAA BBB A BBB`

But *exclude*:
`A B AAAAAA BBBBB`  - breaking at non `.await` points

Conditional *include*/*exclude*:
`BBB AA BBB AAAA A` - **conditional** exclude - violates our code if written in typical serial fashion where A would have to hit a `.await` before it could yield to B.  However, if A & B were designated as non-dependent from the beginning and set as separate 'logic-threads' then it would be an*include*

## Chanels ¿are Dead Drops?

**Warn**: I'm less certain here, just working out how they seem like they *would* work.

"Channels", to me, evokes a sense of one thread or stream pushing info into another.  This is at odds with framework-free / low-framework (gotta kernel: gotta framework) processing -- where the program must animate what is relevant -- it must drive time forward.

So I *assume* "channels" are actually more like DeadDrops. (obfuscation elements of that name arguably appropriate if we consider "channels" unecessarilly opaque)

\*\*\*
Just a linear store of information that multiple threads can write to (with the regular locking and waiting mechanisms that kernels or mutexes handle) and that some other thread is allowed to read and remove from.
\*\*\*
(note: if I have to adorn information mid-way through what I wrote as the relevant bit then I've clearly misorganized my writing)

**Implications/Connections**:
- the commonly seen `mpsc` (multiple producer single consumer) "channel" then makes sense as a common form.  If we assume that (e.g. due to high volume) we don't want to indefinitely persist data in the channel -- instead allowing/defaulting to removal after use -- then having multiple consumers creates a coordination questions.  How many threads must be allowed to see it?  How do we keep track of viewing threads? etc. -- 'single consumer' could be fine across multiple threads if we allow a single, arbitrary consumer to take something (e.g. a task) -- but mpm(viewer) suddenly comes with a lot of nuances and potential misuse without understanding.
        - where multiple producer has, naively, just to deal with contention for a lock to write, perhaps -- and even then, a buffering system could exist to allow free acess to writing, which is then handled ... well ... now we're talking about active management in the context of valuable threads ... maybe not great ..
                - we could also allow writing of threads to heap and putting a pointer in the channel .. not sure what the write rate of, say just u64s would be.  
                        - we could also have even a single bit (not sure if useful given how memory is read) that indicates "am writing" - and if writing objects of known size that should allow multiple simultaneous writes (not sure if useful with current hardware)
- sending somethign in a channel doesn't imply receipt
- sending to a channel can happen after a consumer thread has stopped reading
- tldr: pushing to a channel just makes data available in some variable that could be accessed normally
        - **Question**: as far as async & signalling what kernel or runtime cues are there that something has been added to queue? ('queue cue') 

