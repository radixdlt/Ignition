<div align="center">
  <h1><code>Ignition</code></h1>
  <p>
    <strong>Double the value of your liquidity positions on Radix, earn higher fees on them and enjoy impermanent loss protection.</strong>
  </p>
  <p>
    <a href="https://github.com/radixdlt/Ignition/actions/workflows/test.yml?query=branch%3Amain"><img src="https://github.com/radixdlt/Ignition/actions/workflows/test.yml/badge.svg?query=branch%3Amain" alt="Test Status" /></a>
    <a href="./LICENSE"><img src="https://img.shields.io/github/license/saltstack/salt" alt="License" /></a>
  </p>
</div>

## Introduction

This document covers the technical aspects of Ignition and does not focus so much on the economic or incentives aspect of it beyond the introduction section. If you would like a more detailed explanation of the economic incentives you can find it [here](https://uploads-ssl.webflow.com/6053f7fca5bf627283b582c2/65c3bfd9846b7773b8dd7148_project-Ignition-details.pdf). The hope is that this repository would serve as an example of what a production-ready application written in Scrypto and accompanied by a significant amount of tooling that utilizes the Rust libraries offered in `radixdlt-scrypto` looks like. The discussion around the design decisions present in this document might be relevant and useful to many different applications, perhaps the first that comes to mind is an on-ledger aggregator.

Ignition contains a significant amount of libraries and patterns for writing interfaces, tests, and publishing logic, all of which are covered in this document. Some of these tools are not production-ready or generic enough to be used in other projects, they just meet the needs of Ignition. They're present in this document for completeness and also to introduce the reader to how Ignition dealt with some of the problems it encountered and the tooling addressing them.

## Economical Introduction

In simple terms, project Ignition allows users to provide one side of liquidity and for itself to provide the other side of the liquidity. The protocol is not quite made to be profit-generating for the runner, its main purpose is to incentivize users to provide liquidity by providing users with several benefits:

* The user's contribution is doubled in value as the user contributes one side of the liquidity and Ignition contributes an equal value of the other side of the liquidity.
* Users get some percentage of rewards upfront depending on how long they want their liquidity to be locked up.
* Users have impermanent loss protection and in most cases are guaranteed to withdraw the same amount of resources that they put in plus fees earned on their position.
* Users earn higher fees on their liquidity positions since their position size is doubled.

This makes Ignition a perfect incentive for users who already own an amount of some of the supported resources (on mainnet this is xUSDT, xUSDC, xwBTC, and xETH) and who wish to provide liquidity with a low downside, upfront rewards, increased fees, and impermanent loss protection.

The user locks their resources for some period allowed by the protocol and based on the length of the lockup period they're given some amount of upfront rewards. The longer the lockup period is, the higher the rewards are. When the period is over, the protocol will try to provide the user with the same amount of resources that they put in plus any trading fees earned in the process (on their resource). If that can't be given, then the protocol will try to provide the user with as much of the protocol resource as possible to make them whole in terms of value, capped by the amount of resources obtained when closing the liquidity position.

In Ignition, the term "protocol resource" refers to the resource that Ignition has and that the protocol is willing to lend out to users when they wish to provide liquidity. The term "user resource" refers to the resource that was provided by the user. So, the protocol and user resources are the two sides of the liquidity that go into a liquidity pool. As an example, the mainnet deployment of Ignition uses XRD as the protocol resource and xUSDT, xUSDC, xwBTC, and xETH as the user resources.

## Technical Requirements

Behavioral, economic, and incentive requirements aside, this section discusses the technical requirements of the complete Ignition system.

| Requirement | Description | 
| ----------- | ----------- |
| All aspects of Ignition must be easily upgradable and modifiable. | With the large amount of capital handled by Ignition it must be trivial to upgrade and modify the behavior of the system without needing to wait or rely on a native blueprint upgradeability system to become available in the radix engine. | 
| Ignition must support new exchanges trivially. | It must be trivial for Ignition to support new exchanges that might have not been released at the time of writing Ignition. This means that if Ignition launched with support for exchanges A, B, and C and later on after the release of Ignition a new exchange D was launched then there must be a way for Ignition to support exchange D trivially without needing to move to a new package. |
| Ignition's oracle must be easy to replace. | The oracle used by Ignition must be upgradable such that the oracle doesn't become a single point of failure if an oracle permanently goes out of service or suffers prolonged downtime. Ignition must be able to trivially switch to a new oracle provider at runtime and ideally without the need for a new Ignition package.  | 
| Ignition must not be tied to any protocol resource. | Ignition blueprints should not make the assumption that XRD is the protocol resource since this makes the blueprints difficult to test anywhere where XRD is not in abundance or freely mintable. |
| Ignition must control what pools the users are allowed to contribute to. | Users must not be able to contribute to any pool but only a list of allowed pools. |
| Adding or removing an allowed pool must be trivial. | The addition or removal of an allowed pool must not require a new package and must be simple to do. | 
| The ability for positions to be opened and closed must be configurable. | In the case of a bug being discovered it is important that there are is a way for Ignition to halt operation and not allow for new positions to be opened or closed. Much like the other items enabling or disabling opening or closing of liquidity positions must be trivial and easy to do and must not require a new package. | 
| Each supported exchange should have their own liquidity receipt resource with its own unique data. |  |
| Ignition must be resilient to pool price manipulation attacks and ensure that there is a mechanism to detect such price manipulations and deny service when they happen.| | 

## Architecture

Before discussing the architecture of Ignition it would be good to first look at a key aspect of the system: its need to communicate with the various exchanges, more specifically there are three main invocations that Ignition needs to be able to make to exchanges:

1. When a user opens a liquidity position Ignition needs to be able to call whatever method on the user-selected pool that is used for adding liquidity.
2. When a user closes their liquidity position Ignition needs to be able to call whatever method on the user's pool that removes the liquidity.
3. When the user opens or closes a liquidity position Ignition needs to be able to get the current price as reported by the pool to use for pool price manipulation checks.

The plan for Ignition was to initially integrate with three decentralized exchanges: Ociswap v2, Caviarnine v1, and Defiplaza v2. There does not exist any kind of standardized interface that exchanges need to have, therefore, naturally, all three of these exchanges have completely different interfaces for the three invocations that Ignition needs to make. The following table shows the relevant methods on these blueprints:

<table style="width: 100%" >
<tr>
  <td></td>
  <td><strong>Ociswap v2</strong></td>
  <td><strong>Caviarnine v1</strong></td>
  <td><strong>Defiplaza v2</strong></td>
</tr>
<tr>
<td>Adding Liquidity</td>
<td> 

```rust
fn add_liquidity(
    &mut self,
    left_bound: i32,
    right_bound: i32,
    x_bucket: Bucket,
    y_bucket: Bucket
) -> (Bucket, Bucket, Bucket);
```

</td>
<td>

```rust
fn add_liquidity(
    &mut self,
    tokens_x: Bucket,
    tokens_y: Bucket,
    positions: Vec<(u32, Decimal, Decimal)>,
) -> (Bucket, Bucket, Bucket);
```

</td>
<td>

```rust
fn add_liquidity(
    &mut self,
    input_bucket: Bucket,
    co_liquidity_bucket: Option<Bucket>,
) -> (Bucket, Option<Bucket>);
```

</td>
</tr>
<tr>
<td>Removing Liquidity</td>
<td> 

```rust
fn remove_liquidity(
    &mut self,
    lp_positions: NonFungibleBucket
) -> (Bucket, Bucket);
```

</td>
<td>

```rust
fn remove_liquidity(
    &mut self,
    liquidity_receipt: Bucket
) -> (Bucket, Bucket);
```

</td>
<td>

```rust
fn remove_liquidity(
    &mut self,
    lp_bucket: Bucket,
    is_quote: bool,
) -> (Bucket, Bucket);
```

</td>
</tr>
<tr>
<td>Getting Current Price</td>
<td> 

```rust
fn price_sqrt(
    &self
) -> PreciseDecimal;
```

</td>
<td>

```rust
fn get_price(
    &self
) -> Option<Decimal>;
```

</td>
<td>

N/A

</td>
</tr>
</table>

As mentioned above, the interface of these blueprints differs greatly. Representing this as architecture there are many different ways that this problem of different interfaces can be solved, each of which has its advantages and disadvantages. The following paragraphs discuss some of the ways Ignition could have gone about addressing this problem and then arrive at the final solution that was chosen.

It is good to start with a naive solution to the problem, examine its strengths and weaknesses and then examine how it can be improved. One way that this problem could be solved is by including logic for handling the different exchanges directly into the core Ignition blueprint, the idea is that Ignition would determine which blueprint the passed component belongs to and then there would be branching logic based on the result. If it is an Ociswap v2 pool then it will perform invocations in one way, and similarly with Caviarnine v1 and Defiplaza v2. The best thing about this approach is that it is simple to implement, simple to understand, and simple to audit and check. However, this approach makes upgradeability and the addition of new exchanges very difficult. Each time a new exchange needs to be added or invocation logic needs to be changed then a completely new Ignition package, with the required added support for the new exchange integration or changes to the existing one, is required that changes the logic and migration of funds from the old component to the new component is needed. Additionally, this ties the Ignition protocol itself very closely to the exchanges which is undesirable.

The core problem with the previous naive approach is the fact that the Ignition blueprint became tied to exchanges and their interfaces. The requirement of a new package for any changes that Ignition makes to the supported exchanges or to how the invocations are made is a deal breaker. Therefore since tying them together is undesirable then perhaps splitting them apart would show some desirable characteristics. Therefore, instead of a single blueprint that contains all of the Ignition logic and also the exchange invocations logic, separating them into two separate blueprints or components might be ideal. The first blueprint would be the Ignition protocol blueprint that handles all of the Ignition logic and there would also be an Ignition invocations blueprint that contains all of the code for communicating with the exchanges and it would offer a standardized interface such that the Ignition protocol blueprint becomes fully exchange-agnostic while all of the exchange related logic lives in a separate blueprint. When the Ignition protocol blueprint wishes to invoke a method (e.g., to open a position) on an exchange component it would invoke the Ignition invocations blueprint and that blueprint would handle the invocations accordingly and branching logic accordingly and then return the result. The biggest advantage of this architecture over the previous one is that changing the invocations logic or supporting a new exchange has become significantly simpler, it is as simple as authoring a new Ignition invocations blueprint with whatever changes or additions are required, instantiating a new component from it, and making the Ignition protocol component point to the new invocations component instead of the previous one. Since the invocations blueprint offers a standardized interface then the Ignition protocol component would be able to invoke the new invocations blueprint just fine. The disadvantage of this approach is that the Ignition invocations blueprint is handling many things and this is especially true with how complex the integration logic is. Another disadvantage is that from the point of view of the ledger, any change to the invocation makes the Ignition protocol blueprint point to a completely different invocation blueprint. Meaning that it is difficult to understand from the on-ledger information alone what the change impacts. Therefore, being more granular might be useful.

Finally, this final architecture is the one chosen for Ignition. Building on the previous architecture, splitting up the invocations blueprint into multiple different blueprints that can be changed individually might offer some interesting characteristics. In this architecture there are multiple blueprints: there is exactly one Ignition protocol blueprint and as many exchange adapter blueprints as there are supported exchanges. An exchange [adapter](https://en.wikipedia.org/wiki/Adapter_pattern) is a blueprint with a standardized interface that is known to the Ignition protocol blueprint that handles invocations to exchanges and adapts the pool's interface to the standard expected by the Ignition protocol. This architecture solves the issues of the previous one by splitting the larger blueprint into many different ones providing more granularity when making changes.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./diagrams/architecture-dark.png">
  <img alt="Text changing depending on mode. Light: 'So light!' Dark: 'So dark!'" src="./diagrams/architecture-light.png">
</picture>

The above is a diagram representing the architecture described in the text above. There is a core protocol layer that everything builds on. For an exchange to be integrated an exchange adapter blueprint is written and published, a component is instantiated, and that component is registered into the Ignition protocol as the adapter to use when invocations to a particular exchange are required. There only exists a single adapter component per exchange meaning that if some changes were required to the adapter then all that would change would be the component address of the registered adapter in Ignition.

The Ignition protocol component has a map where it stores a mapping of the `BlueprintId` (that's a package address and a blueprint name) of exchange pools to the following information about the exchange: the liquidity receipt resource to mint when a user opens a position, the component address of the adapter to use for invocations, and the set of component addresses that users are allowed to contribute to. So, for example, if a change is needed to the Ociswap v2 adapter, then the appropriate method will be called on Ignition which would update the component address of the adapter to use on Ociswap v2's entry in this map.

Ignition has a clear separation of concerns between the Ignition protocol and the exchange adapters. The following are the concerns of each of them:

- Ignition Protocol: This contains logic for the management of lockup periods, storing funds, making use of the adapters, position settlement, and everything short of invocations to the actual pools.
- Exchange Adapters: They provide the standardized interface, per exchange, expected by Ignition to add liquidity, remove liquidity, get the pool's current price, and report the current fees.

From this point onward in the document _"Ignition protocol"_ refers to the protocol layer of Ignition which is exchange agnostic and _"Ignition system"_ refers to the system as a whole including the protocol and the various adapters.

One of the core responsibilities of an adapter is to calculate the amount of fees earned on a position between the time that it was opened and closed. In some cases, this calculation might be non-trivial and the adapter might need to store some information about the position when it was first opened somewhere such that it can use this data later on when it closes the position. Since all of the exchanges differ in the way that they work the data that is required to calculate the fees for positions is different between them and does not have a single unified structure. This data is not stored in the adapter's state since that would make upgradeability difficult: moving from one adapter component to another would require migrating this data along with it and if the data is large then this might be difficult to do. Therefore, adapters aim to be as stateless as possible. The only state that it saved on the adapters is state that can either be very cheaply computed or state that must be there. This brings us back to the question: "If such information can not be stored in the adapter's state, then where is it stored?". It is stored in an unstructured field of the liquidity receipt that the user gets back when they open a liquidity position. Thus, when Ignition invokes an adapter for a position to be opened in an exchange the adapter returns the pool units and what the system refers to as the _"adapter-specific information"_. This information is stored in an unstructured field on the `LiquidityReceipt`. It is an unstructured field since all NFTs must have an SBOR schema and each one of the adapters needs to save data of a different structure in this field. Therefore Ignition made this one field unstructured and left the writing, reading, and interpretation of this field to be up to the adapters themselves. Since the adapter has written to this field it also knows how to interpret what it wrote. Making the field unstructured means that this field can take any value, it could be a string, a tuple, an enum, a number, or anything at all. This provides a beautiful abstraction for the data where the higher layer defines the encoding and decoding while the lower layer stores the data not knowing what the data is. This is yet another way that Ignition is built with upgradeability and extensibility in mind with patterns that ensure that moving from one component to another is as seamless as possible.

The fact that there are multiple adapters that Ignition has or that an adapter is used to communicate with the oracle or the pools is abstracted away from users of Ignition since the main component that users of Ignition need to invoke is the Ignition protocol component.

The table below examines how Ignition addresses its [technical requirements](#technical-requirements):

| Requirement | Addressed By | 
| ----------- | ----------- |
| All aspects of Ignition must be easily upgradable and modifiable. | In this architecture all aspects are indeed upgradable and modifiable. The adapters uses for any of the exchanges can be changed easily and fund migration to the new component is trivial. | 
| Ignition must support new exchanges trivially. | For a new exchange to be supported it needs an entry in the Ignition exchange integrations map with the adapter to use, the liquidity receipt resource to mint and expect, and the set of all allowed pools. No new package is required, modifications can be made at runtime. |
| Ignition's oracle must be easy to replace. | The oracle integration into Ignition assumes a standardized oracle adapter interface. This means that the oracle can be replaced by simply writing an adapter for the a new oracle provider and making Ignition point to the oracle. | 
| Ignition must not be tied to any protocol resource. | Ignition's protocol resource is stored in its state and must be specified when the component is first instantiated. |
| Ignition must control what pools the users are allowed to contribute to. | Ignition's map that stores the exchange integrations information stores this data and Ignition disallows contributions to any pool that is not one of the allowed pools. |
| Adding or removing an allowed pool must be trivial. | It is as simple as finding the entry in the exchange integrations map and mutating the set of allowed pools to add or remove one. | 
| The ability for positions to be opened and closed must be configurable. | Ignition has flags that control if positions can be opened or closed. | 
| Each supported exchange should have their own liquidity receipt resource with its own unique data. | The exchange integrations map stores the liquidity receipt resource of each exchange and this is the resource that it mints and this is the resource that it expects when closing the liquidity position. |
| Ignition must be resilient to pool price manipulation attacks and ensure that there is a mechanism to detect such price manipulations and deny service when they happen.| Ignition uses a price oracle to ensure that the price of the pool is within an allowed price difference from the oracle. | 

## Interfaces

As can be seen in the [architecture](#architecture) section standardized interfaces are a core aspect of the Ignition system that it can not function without. However, there does not currently exist any tooling in the Scrypto toolchain for using interfaces. To be more specific, Ignition's needs with regards to interfaces are:

1. The need to be able to define standardized interfaces. This will be used to define the interface of the Pools and Oracle that Ignition invokes.
2. The need for blueprints to implement the defined standardized interface. This will be used in implementing exchange adapters so that they adhere to the defined interface.
3. The ability to type-safely call methods on a component that implements the interface. This will be needed when Ignition invokes a component that it believes to implement some interface.

One of the core libraries developed for Ignition is the [`scrypto-interface`](./libraries/scrypto-interface/) library which is built to address this exact problem of interfaces and the need for them. This library focuses on users who are more interested in the interface than the implementation, which is commonly referred to as a "has a" versus "is a". The table below explains how the [`scrypto-interface`](./libraries/scrypto-interface/) library addresses the needs that Ignition has.

| Ignition Need | How [`scrypto-interface`](./libraries/scrypto-interface/) Addresses It
| ---- | ---- |
| The need to be able to define standardized interfaces. This will be used to define the interface of the Pools and Oracle that Ignition invokes. | The library has a macro for defining an interface called the `define_interface` macro which generates out a trait based on the interface and Scrypto, Scrypto-Test, and Manifest Builder stubs. | 
| The need for blueprints to implement the defined standardized interface. This will be used in implementing exchange adapters so that they adhere to the defined interface. | The trait that is generated by the `define_interface` macro can be implemented on blueprints with the aid of the `blueprint_with_traits` macro which is a drop-in replacement for the `blueprint` macro that extends it to allow for traits to be implemented blueprints. This provides compile-time checking of trait implementations and would provide compile-time errors in the event that the interface implementation deviates from the interface definition. | 
| The ability to type-safely call methods on a component that implements the interface. This will be needed when Ignition invokes a component that it believes to implement some interface. | The `define_interface` macro generates stubs for Scrypto, Scrypto Test, and the Manifest Builder which offers a type-safe way to invoke components implementing the interface regardless of the environment. | 

More information on the [`scrypto-interface`](./libraries/scrypto-interface/) library can be found in its [`README.md`](./libraries/scrypto-interface/README.md). Usage of the [`scrypto-interface`](./libraries/scrypto-interface/) library can be found in the [`libraries/ports-interface/src/`](./libraries/ports-interface/src/) which contains the interface definitions of the pool and oracle adapters.

Based on the separation of concerns defined between the protocol layer and the adapters in the [architecture](#architecture) section the following interface is defined for the pool and oracle adapters.

### Pool Adapter Interface

```rust
define_interface! {
    PoolAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput;
        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Vec<Bucket>,
            adapter_specific_information: AnyValue
        ) -> CloseLiquidityPositionOutput;
        fn price(&mut self, pool_address: ComponentAddress) -> Price;
        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress
        ) -> (ResourceAddress, ResourceAddress);
    }
}

#[derive(Debug, ScryptoSbor)]
pub struct OpenLiquidityPositionOutput {
    pub pool_units: IndexedBuckets,
    pub change: IndexedBuckets,
    pub others: Vec<Bucket>,
    pub adapter_specific_information: AnyValue,
}

#[derive(Debug, ScryptoSbor)]
pub struct CloseLiquidityPositionOutput {
    pub resources: IndexedBuckets,
    pub others: Vec<Bucket>,
    pub fees: IndexMap<ResourceAddress, Decimal>,
}

#[sbor(transparent)]
pub struct AnyValue((ScryptoValue,));

#[derive(Debug, ScryptoSbor)]
pub struct IndexedBuckets(IndexMap<ResourceAddress, Bucket>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, ScryptoSbor)]
pub struct Price {
    pub base: ResourceAddress,
    pub quote: ResourceAddress,
    pub price: Decimal,
}
```

The interface of the pool adapter is quite small and is made up of the exact set of methods that Ignition needs to operate so it includes methods for opening a liquidity position, closing it, getting the current price, and getting the resource addresses of the resources in the pool. 

For the Ignition protocol to open a liquidity position on some pool it needs to call the `open_liquidity_position` method on the adapter for that exchange and all that it needs to supply is the address of the pool and the two buckets containing the two sides of the liquidity. The adapter would open the liquidity position and return an `OpenLiquidityPositionOutput` object containing the following:

1. `pool_units` - The pool units obtained from opening this liquidity position. These could be native pool units if the exchange uses them or could be the exchange's custom pool units. No assumptions are made in that regard by the protocol but the adapter itself understands what their type is since the adapter is aware of the exchange's implementation details.
2. `change` - Any change that remains in the input buckets that was not consumed when the liquidity position was opened.
3. `others` - Any other buckets that were returned by the exchange when the liquidity position was opened. This is currently not in use by any of the existing integrations but the Ignition protocol understands this field to be "buckets that should be returned to the caller without any processing" which could be used for exchanges that provide rewards when liquidity positions are opened.
4. `adapter_specific_information` - As discussed in the [architecture](#architecture) section, adapters do not store position data in their state to make upgradeability easier, they instead store that data in an unstructured field on the `LiquidityReceipt` resource minted for users when they open a liquidity position such that a higher layer (the adapter) understands the encoding and decoding and a lower layer (the protocol) handles the storage of the data without doing any interpretation of it. The type used for this is `AnyValue` which as the name suggests, could take any value.

When closing a liquidity position the Ignition protocol calls the `close_liquidity_position` method on the adapter for that exchange supplying the address of the pool, the pool units, and the adapter-specific information that the adapter wanted the protocol to store when the position was first opened. It is the responsibility of this method to close the liquidity position and to calculate the fees earned on the position between the time for the period that it was open. Therefore, this method returns a `CloseLiquidityPositionOutput`` object containing the following:

1. `resources` - The resources obtained when closing the liquidity position. Typically this would be the protocol and user resources.
2. `others` - Any other buckets that were returned by the exchange when the liquidity position was opened. This is currently not in use by any of the existing integrations but the Ignition protocol understands this field to be "buckets that should be returned to the caller without any processing" which could be used for exchanges that provide rewards when liquidity positions are closed.
3. `fees` - The fees that the adapter believes were earned on the position for the period that it was opened. 

Other methods on the pool adapter are simpler and are pretty much just getters in most cases, although they could also perform some computations. The `price` gets the current price of the pair as reported by the pool and the `resource_addresses` method gets the address of the two resources that make up the pool.

As can be observed from this section, the Ignition protocol does not need to have any knowledge whatsoever of what method needs to be called on exchanges to get this information, what arguments it takes, or what the returns are. All of this is abstracted away from the protocol through the adapters which include all of this logic and expose it through a simple and minimal interface.

### Oracle Adapter Interface

```rust
define_interface! {
    OracleAdapter {
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> (Decimal, Instant);
    }
}
```

The interface of the oracle adapter is significantly simpler than that of the pools. There is just a single `get_price` method that the Ignition protocol can invoke with the address of the base and quote resource and it returns a tuple of the price and the time when the price was submitted to the oracle.

Currently, Ignition is using an oracle built in-house. However, the fact that Ignition does not invoke the oracle directly and invokes an adapter instead means that Ignition can work with any oracle provider due to this one layer of indirection. This means that when Supra Oracle launches or when other oracle providers launch Ignition can switch to using them as an oracle provider instead of our in-house oracle with no downtime or new package.

## Prototyping

Prototyping is essential to answer the question of whether what is proposed is technically possible or not. For a while before the announcement of Ignition prototypes of it were being built behind closed doors. However, some factors made it challenging to build these prototypes. This section discusses these challenges and how Ignition overcame them.

### Package Dependencies

To run a prototype of Ignition either all or some of the Ociswap, Caviarnine, and Defiplaza packages are required. Of course, these packages do not come in a default instantiation of the `TestRunner` or the `TestEnvironment` since they are third-party packages. Additionally, all of these exchanges are closed source (except for DefiPlaza) which means that the package can not be compiled from source. In light of that, how could a prototype be built that integrates Ignition with these exchanges if their packages are not available for local testing?

These packages are actually all available in one place: on the ledger. One way to get those packages and use them for local testing is to dump their substates from ledger and then flash them onto the substate database of a local test database that can then be used with the `TestRunner` or the `TestEnvironment`.

The [`package-dumper`](./tools/package-dumper/) tool does precisely that: given the path to the database of a node and a package address it recursively finds all substates that the package needs and dumps them to the file system. The dumped file could then be flashed onto a local database making these packages locally available.

The following is an example usage of this tool to demonstrate how to get the exchange packages locally to do testing:

The first step would be to use the [`package-dumper`](./tools/package-dumper/) tool to dump the state into the local file system. This example dumps the packages of Ociswap, Caviarnine, and Defiplaza.

```
(
    export STATE_MANAGER_DATABASE_PATH="/etc/babylon-ledger/state_manager/state_manager/";
    export OCISWAP_PACKAGE="package_rdx1pkrgvskdkglfd2ar4jkpw5r2tsptk85gap4hzr9h3qxw6ca40ts8dt";
    export CAVIARNINE_PACKAGE="package_rdx1p4r9rkp0cq67wmlve544zgy0l45mswn6h798qdqm47x4762h383wa3";
    export DEFIPLAZA_PACKAGE="package_rdx1p4dhfl7qwthqqu6p2267m5nedlqnzdvfxdl6q7h8g85dflx8n06p93";
    cargo run --package package-dumper -- $STATE_MANAGER_DATABASE_PATH $OCISWAP_PACKAGE $CAVIARNINE_PACKAGE $DEFIPLAZA_PACKAGE
)
```

Note that the above might take some time to run since it will find all substates of the referenced packages and will recursively do that for each discovered node. For packages with static dependencies (such as the Caviarnine package), this would take a while to run.

After the [`package-dumper`](./tools/package-dumper/) is done and the package dump is outputted to the file system the packages can then be flashed to any substate database and used with the `TestRunner` or the `TestEnvironment`. The following example shows how that can be done:

```rust
const PACKAGES_BINARY: &'static [u8] = include_bytes!(
    concat!(env!("OUT_DIR"), "/uncompressed_state.bin")
);

let (addresses, database_updates) =
    scrypto_decode::<(Vec<PackageAddress>, DatabaseUpdates)>(PACKAGES_BINARY)
        .expect("Can't fail!");

let ociswap_package = addresses[0];
let caviarnine_package = addresses[1];
let defiplaza_package = addresses[2];

let mut test_runner = {
    let mut in_memory_substate_database =
        InMemorySubstateDatabase::standard();
    in_memory_substate_database.commit(&database_updates);
    TestRunnerBuilder::new()
        .with_custom_database(in_memory_substate_database)
        .without_trace()
        .build()
};
```

At this point, the local `test_runner` has the Ociswap, Caviarnine, and Defiplaza packages all available and ready to use in local testing!

### Interfaces

With packages now available for local testing comes a new challenge: what blueprints, methods, and functions are available on those packages to call? Or, what is the interface of those packages?

As previously mentioned, most of these packages are closed source which means that the interface can not be found that way. For this, Ignition used a tool from `radixdlt-scrypto` called `scrypto-bindgen` which is capable of generating the interface of packages on ledger. 

With the packages available locally and the interface known this allowed Ignition to be prototyped and facilitated that phase.

## Testing

Ignition uses the existing testing frameworks and also has several testing techniques that are unique to it. All tests are available in the [`tests`](./testing/tests/) crate.

The `TestEnvironment` is primarily used to write unit tests for Ignition and to test units in isolation of the system. As an example, this framework is used for smoke tests like testing that positions can be opened and closed and unit tests such as that the fees reported are correct, the pool units from opening a position are correct, and so on.

The `TestRunner` is used for integration tests or to test how Ignition would function when invoked in a transaction. This is used in tests such as ensuring that positions can be opened and closed within the fee limit.

Since the system under test in all of the tests is always going to be Ignition the testing framework for Ignition comes with a way of initializing the Ignition environment to start testing straight away without needing to manually publish, bootstrap, and configure the environment. It is as simple as:

```rust
// Instantiates a new environment that uses the `TestEnvironment`.
let scrypto_test_env = ScryptoTestEnv::new().unwrap();

// Instantiates a new environment that uses the `TestRunner`.
let scrypto_unit_env = ScryptoUnitEnv::new();
```

Certain issues might not appear when testing complex projects like Ignition against an ideal environment with a controlled set of resources, pools, and entities in general. However, these issues might appear when testing against a running environment. Ideally, the environment that we want to test against is the environment that we will be deploying to, which is mainnet. However, this is impractical for several reasons:

1. Costly - Tests in any modern software project are run locally, on push, and in PRs. If each test costs XRD or some user resources then each test run would be expensive and the overall cost of development would be high. Additionally, it would impede development as tests would be run less often. In addition to that, if Ignition were to be tested against real pools then this would add even more cost. If it were to be tested with dummy resources that are all freely mintable then it's back to square one and it is being tested in an ideal environment with ideal pools.
2. Manual - Automating such tests might be difficult and running them manually is error-prone and inconvenient as these tests should ideally run in CI on each push.
3. Degree of Control - Tests sometimes require a larger degree of control over the environment to effectively test. As an example, many of the Ignition tests for opening and closing positions push the time forward after opening a position. This certainly can not be done on mainnet or any test network for that matter which makes testing more difficult.

Therefore, the simple approach of publishing and bootstrapping Ignition to mainnet and then making several transactions that test it can not be done. For this, Ignition uses an innovative approach to tests called "stateful tests" which allows Ignition to be tested against mainnet state without making any transactions to mainnet and without costing anything. In simple terms, Ignition's approach to stateful testing gives the test a `TestRunner` that has mainnet state in full. It follows the following logic: since the node has a substate database and since the `TestRunner` can be configured to run against any substate database then we can have a `TestRunner` running against the database of mainnet. To be explicitly clear, this approach requires a running node with a fully synced database.

Ignition also ensures that the node's database does not get corrupted by all of the database commits that the `TestRunner` will perform by using a `SubstateDatabaseOverlay` which is a database wrapper that provides an overlay that all writes and deletes go to and that reads are preferred from. This means that the database does not need to be mutated or written to at all when used with the `TestRunner` and will not be corrupted, all the data will be written to the in-memory overlay.

This approach to testing proved to be useful in many ideas, but especially in ensuring that all Ignition positions can be opened and closed within the allowed fee limit. Testing this on local pools did not show any issues but this issue showed itself in stateful tests where the pools are not ideal.

The following is an example of what a test against mainnet state looks like:

```rust
// We do not use the `test` attribute macro but use a `mainnet_test` declarative
// macro instead.
#[apply(mainnet_test)]
fn example_test(
    // The test function takes a few arguments which are supplied by the "test
    // harness" which is the notary's account, Ignition's publishing receipt,
    // and the test runner to use. All other details involving the database and
    // node are completely abstracted away and the test runner provided here as
    // an argument can be used just like any other test runner. This test can 
    // also be run just like any test and would not be treated in any special 
    // way by cargo or the compiler. 
    notary: AccountAndControllingKey,
    receipt: &PublishingReceipt,
    test_runner: &mut StatefulTestRunner<'_>,
) {
    todo!()
}
```

## Publishing and Bootstrapping

Ignition is made up of multiple packages, components, and resources which makes the process to follow to get to a running system quite long. Additionally, the modular nature of Ignition means that the process might not always be the same. As an example, one time the Ignition, Ociswap Adapter, Caviarnine Adapter, and Defiplaza Adapter packages might be needed while at other times only the Ignition package might need to be published since it was the only part that changed. The process of publishing and bootstrapping Ignition must be easy and efficient to allow for quick iterations and to quickly be able to get Ignition in the hands of integrators.

The typical approach of using the developer console for this would have been too error-prone, manual, lengthy, time-consuming, and inefficient. For this Ignition has a tool called the [`publishing-tool`](./tools/publishing-tool/) which is a tool written specifically for Ignition to allow users of the tool to declaratively define what should be done when publishing and for the tool to do the publishing and everything else.

The following are the features offered by the [`publishing-tool`](./tools/publishing-tool/):

1. Handles the entirety of the complex publishing process of Ignition from beginning to end. 
1. Handles all of the two-way linking for all of the created entities.
1. Allows the user of the tool to specify and configure it in a declarative way that is higher level. The tool then translates the configuration into transaction manifests and transactions.
1. Works against any network that has a Gateway API.
1. Has the ability to simulate the entire publishing process before it is run. This allows for issues in the configuration to be detected before spending any XRD on the actual publication and failing in the middle.
1. The output of the publishing tool is a publishing receipt with the entire configuration of Ignition and the address of the created entities.
1. Batches up package publishing to save on fees.

The term declarative configuration means that the user of the tool does not need to write any manifests on their own. Instead, they declare their intent and the tool will translate it into the appropriate set of manifests required to achieve the user's intent. An example of one of those configurations can be found in [`tools/publishing-tool/src/configuration_selector/mainnet_production.rs`](tools/publishing-tool/src/configuration_selector/mainnet_production.rs) module which is also seen below:

```rust
pub fn mainnet_production(
    notary_private_key: &PrivateKey,
) -> PublishingConfiguration {
    let notary_account_address =
        ComponentAddress::virtual_account_from_public_key(
            &notary_private_key.public_key(),
        );

    PublishingConfiguration {
        protocol_configuration: ProtocolConfiguration {
            protocol_resource: XRD,
            user_resource_volatility: UserResourceIndexedData {
                bitcoin: Volatility::Volatile,
                ethereum: Volatility::Volatile,
                usdc: Volatility::NonVolatile,
                usdt: Volatility::NonVolatile,
            },
            reward_rates: indexmap! {
                LockupPeriod::from_months(9).unwrap() => dec!(0.125),
                LockupPeriod::from_months(10).unwrap() => dec!(0.145),
                LockupPeriod::from_months(11).unwrap() => dec!(0.17),
                LockupPeriod::from_months(12).unwrap() => dec!(0.2),
            },
            allow_opening_liquidity_positions: false,
            allow_closing_liquidity_positions: false,
            maximum_allowed_price_staleness_in_seconds: 60,
            maximum_allowed_price_difference_percentage: dec!(0.05),
            entities_metadata: Entities {
                protocol_entities: ProtocolIndexedData {
                    ignition: metadata_init! {
                        "name" => "Ignition", updatable;
                        "description" => "The main entrypoint into the Ignition liquidity incentive program.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    simple_oracle: metadata_init! {
                        "name" => "Ignition Oracle", updatable;
                        "description" => "The oracle used by the Ignition protocol.", updatable;
                        "tags" => vec!["oracle"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                },
                exchange_adapter_entities: ExchangeIndexedData {
                    ociswap_v2: metadata_init! {
                        "name" => "Ignition Ociswap v2 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol for Ociswap v2 interactions.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    defiplaza_v2: metadata_init! {
                        "name" => "Ignition DefiPlaza v2 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol for DefiPlaza v2 interactions.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    caviarnine_v1: metadata_init! {
                        "name" => "Ignition Caviarnine v1 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol for Caviarnine v1 interactions.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                },
            },
        },
        dapp_definition_metadata: indexmap! {
            "name".to_owned() => MetadataValue::String("Project Ignition".to_owned()),
            "description".to_owned() => MetadataValue::String("A Radix liquidity incentives program, offered in partnership with select decentralized exchange dApps in the Radix ecosystem.".to_owned()),
            "icon_url".to_owned() => MetadataValue::Url(UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"))
        },
        transaction_configuration: TransactionConfiguration {
            notary: clone_private_key(notary_private_key),
            fee_payer_information: AccountAndControllingKey::new_virtual_account(
                clone_private_key(notary_private_key),
            ),
        },
        badges: BadgeIndexedData {
            oracle_manager_badge: BadgeHandling::CreateAndSend {
                account_address: component_address!(
                    "account_rdx168nr5dwmll4k2x5apegw5dhrpejf3xac7khjhgjqyg4qddj9tg9v4d"
                ),
                metadata_init: metadata_init! {
                    "name" => "Ignition Oracle Manager", updatable;
                    "symbol" => "IGNOM", updatable;
                    "description" => "A badge with the authority to update the Oracle prices of the Ignition oracle.", updatable;
                    "tags" => vec!["badge"], updatable;
                    // Dapp definition will be automatically added by the
                    // publisher accordingly.
                },
            },
            protocol_manager_badge: BadgeHandling::CreateAndSend {
                account_address: notary_account_address,
                metadata_init: metadata_init! {
                    "name" => "Ignition Protocol Manager", updatable;
                    "symbol" => "IGNPM", updatable;
                    "description" => "A badge with the authority to manage the Ignition protocol.", updatable;
                    "tags" => vec!["badge"], updatable;
                    // Dapp definition will be automatically added by the
                    // publisher accordingly.
                },
            },
            protocol_owner_badge: BadgeHandling::CreateAndSend {
                account_address: notary_account_address,
                metadata_init: metadata_init! {
                    "name" => "Ignition Protocol Owner", updatable;
                    "symbol" => "IGNPO", updatable;
                    "description" => "A badge with owner authority over the Ignition protocol.", updatable;
                    "tags" => vec!["badge"], updatable;
                    // Dapp definition will be automatically added by the
                    // publisher accordingly.
                },
            },
        },
        user_resources: UserResourceIndexedData {
            bitcoin: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1t580qxc7upat7lww4l2c4jckacafjeudxj5wpjrrct0p3e82sq4y75"
                ),
            },
            ethereum: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1th88qcj5syl9ghka2g9l7tw497vy5x6zaatyvgfkwcfe8n9jt2npww"
                ),
            },
            usdc: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf"
                ),
            },
            usdt: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw"
                ),
            },
        },
        packages: Entities {
            protocol_entities: ProtocolIndexedData {
                ignition: PackageHandling::LoadAndPublish {
                    crate_package_name: "ignition".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Package", updatable;
                        "description" => "The implementation of the Ignition protocol.", updatable;
                        "tags" => Vec::<String>::new(), updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "Ignition".to_owned(),
                },
                simple_oracle: PackageHandling::LoadAndPublish {
                    crate_package_name: "simple-oracle".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Simple Oracle Package", updatable;
                        "description" => "The implementation of the Oracle used by the Ignition protocol.", updatable;
                        "tags" => vec!["oracle"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "SimpleOracle".to_owned(),
                },
            },
            exchange_adapter_entities: ExchangeIndexedData {
                ociswap_v2: PackageHandling::LoadAndPublish {
                    crate_package_name: "ociswap-v2-adapter-v1".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Ociswap v2 Adapter Package", updatable;
                        "description" => "The implementation of an adapter for Ociswap v2 for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "OciswapV2Adapter".to_owned(),
                },
                defiplaza_v2: PackageHandling::LoadAndPublish {
                    crate_package_name: "defiplaza-v2-adapter-v1".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition DefiPlaza v2 Adapter Package", updatable;
                        "description" => "The implementation of an adapter for DefiPlaza v1 for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "DefiPlazaV2Adapter".to_owned(),
                },
                caviarnine_v1: PackageHandling::LoadAndPublish {
                    crate_package_name: "caviarnine-v1-adapter-v1".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Caviarnine v1 Adapter Package", updatable;
                        "description" => "The implementation of an adapter for Caviarnine v1 for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "CaviarnineV1Adapter".to_owned(),
                },
            },
        },
        exchange_information: ExchangeIndexedData {
            ociswap_v2: None,
            defiplaza_v2: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4dhfl7qwthqqu6p2267m5nedlqnzdvfxdl6q7h8g85dflx8n06p93"
                    ),
                    blueprint_name: "PlazaPair".to_owned(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpv5g5a86qezw0g46w2ph8ydlu2m7jnzxw9p4lx6593qn9fmnwerta"
                        ),
                    },
                    ethereum: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1crwdzvlv7djtkug9gmvp9ejun0gm0w6cvkpfqycw8fcp4gg82eftjc"
                        ),
                    },
                    usdc: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpw85pmjl8ujjq7kp50lgh3ej5hz3ky9x65q2cjqvg4efnhcmfpz27"
                        ),
                    },
                    usdt: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1czr2hzfv2xnxdsts4a02dglkn05clv3a2t9uk04709utehau8gjv8h"
                        ),
                    },
                },
                liquidity_receipt: LiquidityReceiptHandling::CreateNew {
                    non_fungible_schema:
                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                            LiquidityReceipt<AnyValue>,
                        >(),
                    metadata: metadata_init! {
                        "name" => "Ignition LP: DefiPlaza", updatable;
                        "description" => "Represents a particular contribution of liquidity to DefiPlaza through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                        "tags" => vec!["lp token"], updatable;
                        "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"), updatable;
                        "DEX" => "DefiPlaza", updatable;
                        "redeem_url" => UncheckedUrl::of("https://radix.defiplaza.net/ignition"), updatable;
                    },
                },
            }),
            caviarnine_v1: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4r9rkp0cq67wmlve544zgy0l45mswn6h798qdqm47x4762h383wa3"
                    ),
                    blueprint_name: "QuantaSwap".to_owned(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cp9w8443uyz2jtlaxnkcq84q5a5ndqpg05wgckzrnd3lgggpa080ed"
                        ),
                    },
                    ethereum: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpsvw207842gafeyvf6tc0gdnq47u3mn74kvzszqlhc03lrns52v82"
                        ),
                    },
                    usdc: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cr6lxkr83gzhmyg4uxg49wkug5s4wwc3c7cgmhxuczxraa09a97wcu"
                        ),
                    },
                    usdt: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cqs338cyje65rk44zgmjvvy42qcszrhk9ewznedtkqd8l3crtgnmh5"
                        ),
                    },
                },
                liquidity_receipt: LiquidityReceiptHandling::CreateNew {
                    non_fungible_schema:
                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                            LiquidityReceipt<AnyValue>,
                        >(),
                    metadata: metadata_init! {
                        "name" => "Ignition LP: Caviarnine", updatable;
                        "description" => "Represents a particular contribution of liquidity to Caviarnine through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                        "tags" => vec!["lp token"], updatable;
                        "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"), updatable;
                        "DEX" => "Caviarnine", updatable;
                        "redeem_url" => UncheckedUrl::of("https://www.caviarnine.com/ignition"), updatable;
                    },
                },
            }),
        },
        additional_information: AdditionalInformation {
            ociswap_v2_registry_component_and_dapp_definition: None,
        },
        additional_operation_flags: AdditionalOperationFlags::empty(),
    }
}
```

The following is an example command used to publish and bootstrap Ignition on stokenet. The private key seen in this command is an example private key created just for this example to make it easier for you to run:

```
cargo run --package publishing-tool -- publish stokenet-testing 1eb34aa0be9c78e450a9f2eed3702e5109b21484671566e67cbf173e2c45942a
```

The main part of the code that translates the declarative configuration into manifests and submits them to the network is [`tools/publishing-tool/src/publishing/handler.rs`](./tools/publishing-tool/src/publishing/handler.rs).