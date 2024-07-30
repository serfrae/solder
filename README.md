# Solder - Solana Data Aggregator

## Design
The design went through a couple of iterations and there were a couple of troubles
I had implementing the solution due to some erroneous assumptions and time constraints.

One of the primary issues I had was retrieving block data. To be clear, retrieving
block data is trivial via an RPC API call but this seemed like a rather slow approach
if we wanted real time data. I had initially opted for a websocket subscription as detailed
in the (solana) [ https://solana.com/docs/rpc/websocket/blocksubscribe ] however after writing
the implementation it would seem that this method is not typically available on all RPC providers
or is available at a higher pricing tier, i.e. Business or above. My second approach made a rather
erroneous assumption based on outdated knowledge that number of tokens transferred and the number of
SOL transferred between accounts appeared on the Logs, this seems to no longer be the case.

So the solution that is implemented in this program opts to subscribe to slots. It should be noted
that often the slot that is emitted by the subscription may not have a block associated with it
or at least that was what preliminary testing showed, so I opted to retrieve blocks from the parent
slot.

This approach requires making an API call every time a new slot is received.
Testing on my system showed that 5 API worker clients were required to drain the "slot queue".

Secondly while processing this data, I also made an erroneous assumption that signatures of all the transactions
in the block would be included in the data - this was incorrect and it seemed that more often than not, the 
`signatures` field, which I assume is meant to be an array of the signatures for the transaction, is empty.
This makes it difficult to seperate the data into different tables of a database making storing the data
harder than it needs to be if we are to use a relational database, which I had opted for with Postgres
thinking it would be perfect for this sort of data, mostly utilising the signature as the primary key.

Nevertheless the design uses multiple generic worker pools for each layer of the application, 
each given its own thread pool so that configuration can be fine-tuned to the requirements of the user. 
Each worker pool is, in a very abstract sense, managed by a `worker_manager` that 
is mostly just responsible for initialising the workers, passing them their respective 
queues and handling shutdown via a worker's join handle. The request worker manager
has a slightly different implementation as it has a `monitor` trait implemented, 
and in the way it's workers are initialised. While the other worker managers 
just initialise the maximum number of workers allowed by the number of worker 
threads passed into it, the `request_worker_manager` takes additional configuration options. 
These configuration options define the minimum number of workers and a `scale_up_threshold` 
and `scale_down_threshold` so that more workers can be created and destroyed on 
demand depending on the number of incoming requests. I opted for this difference 
in implementation as the stream of data and processing time are relatively constant 
on the retrieval side but can vary wildly on the request side. It therefore made 
sense to create a worker pools for each layer on thethat was last week retrieval side that could be 
fine-tuned as the workers would essentially remain busy throughout the application's 
run time. Whereas, on the retrieval side the workers could spend time idling and 
consuming resources if too many are created, or a bottleneck if too few are created,
thus a scaling approach to their number seemed to most appropriate. The manager 
on the retrieval side simply scales up the number of workers by (TODO: N) if there 
is a backlog of requests in the queue, and scales down the number of workers if a 
worker is idle for (n time). This is simply implemented by having the worker track 
how long it is idle for and setting a flag, the manager will clean-up workers that 
have this flag, ensuring that the minimum number of workers specified by the configuration 
are always available by checking the length of the `Vec` that holds the worker's join handle.

The implementation is highly extensible, composable, and ensures strict seperation of concerns. 
None of the main program logic needs to be altered to accomodate new types instead, 
type specific logic is implemented through the use of traits. More than one data type 
can be easily retrieved and inserted into the database by instatiating a set of components 
for that data type, thus greatly reducing code duplication and ensuring the codebase is
easily maintainable. Furthermore new layers are easily added by modifying the receiver of data. 
For example, in the first implementation, where I attempted to subscribe to block updates, no RPC client
was required and block data was sent directly to the corresponding processing worker pool.
In order to accomodate the RPC client I added another worker pool for the RPC client that
and moved the receiver from the processing worker to the rpc client and provided a different
queue to link those two components.

### Restraints
While writing this I admit I ran out of time, I had started before I fell severely ill,
and attempted to complete it during the time that I was, the prevalent brain fog during
this period made it frustrating to work through some of the issues and inevitably
meant I could not write tests or handle errors in a correct manner which is something I'm
quite disappointed about.

### Trepidations
I initially had some issues with mpsc::channels that I couldn't discern the error for,
and opted to use crossbeam queues as a replacement - I wonder if this was the right choice
as there doesn't seem to be enough data in each queue to warrant the use of SegQueues,
I would probably revisit this at a later date.

Secondly the nature of the data from the RPC providers did seem to be incomplete at time
making designing a structure that could be split into various tables and retrieved easily
somewhat difficult. This mostly revolved around signatures. I fully expected transaction
signatures to be available with a confirmed block (`UiConfirmedBlock`) but this array
was often empty, meaning it would be hard to segregate the transaction data from the block
data and subsequently retrieve it if it was. This also seemed to be the issue with "pre-balances" and "post-balances".

### More Time
If I had more time I would've liked to implement an API on top of the library so that
in the event I were to make this open source and a crate, there would be a simple
interface for other users. As it stands, I find the code a little unwieldy in `main.rs`
unless someone knows exactly what they are doing and want from this library, although
the general structure of it is easy to understand.

## Setup
Your system will require a postgres instance setup and running, as of this writing,
the only table implemented is the "blocks" table but it is still sufficient to retrieve
all the data - accounts, transactions, and blocks with time parameters.
I have provided the format for the block table in a `.psql` file that can be 
run in psql to create the table. After which you may input your configurations into the
Config.toml provided and the program should run. I do not recommend running this with `RUSTLOG=info`,
or higher, as `debug` will output tokio's debug text which makes the output too noisy.


