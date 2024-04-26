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

This document covers the technical aspects of Ignition and does not focus so much on the economic or incentives aspect of it beyond the introduction section. If you would like a more detailed explanation of the economic incentives you can find it [here](https://uploads-ssl.webflow.com/6053f7fca5bf627283b582c2/65c3bfd9846b7773b8dd7148_project-Ignition-details.pdf). The hope is that this repository would serve as an example of what a production-ready application written in Scrypto and accompanied by a significant amount of tooling that utilizes the Rust libraries offered in `radixdlt-scrypto` looks like. Ignition contains a significant amount of libraries and patterns for writing interfaces, tests, and publishing logic, all of which are covered in this document.

If you are writing a production application there are a number of sections in this document that might be of interest to you such as the [Testing](#testing) and [Publishing and Bootstrapping](#publishing-and-bootstrapping) sections which provide information on what was done for Ignition in those areas.

In simple terms, project Ignition allows users to provide one side of liquidity and for itself to provide the other side of the liquidity. The protocol is not quite made to be profit-generating, its main purpose is to incentivize users to provide liquidity by providing users with several benefits:

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
| All aspects of Ignition must be easily upgradable and modifiable | With the large amount of capital handled by Ignition it must be trivial to upgrade and modify the behavior of the system without needing to wait or rely on a native blueprint upgradeability system to become available in the radix engine. | 
| Ignition must support new exchanges trivially. | It must be trivial for Ignition to support new exchanges that might have not been released at the time of writing Ignition. This means that if Ignition launched with support for exchanges A, B, and C and later on after the release of Ignition a new exchange D was launched then there must be a way for Ignition to support exchange D trivially without needing to move to a new package. |
| Ignition's oracle must be easy to replace. | Ignition uses a price oracle for reasons that we get into later in this document. The oracle used by Ignition must be upgradable such that the oracle doesn't become a single point of failure if an oracle permanently goes out of service or suffers prolonged downtime. Ignition must be able to trivially switch to a new oracle provider at runtime and ideally without the need for a new Ignition package.  | 
| Ignition must not be tied to any protocol resource. | Ignition blueprints should not make the assumption that XRD is the protocol resource since this makes the blueprints difficult to test anywhere where XRD is not in abundance or freely mintable. |
| Ignition must control what pools the users are allowed to contribute to. | Users must not be able to contribute to any pool but only a list of allowed pools. |
| Adding or removing an allowed pool must be trivial | The addition or removal of an allowed pool must not require a new package and must be simple to do. | 
| The ability for positions to be opened and closed must be configurable. | In the case of a bug being discovered it is important that there are is a way for Ignition to halt operation and not allow for new positions to be opened or closed. Much like the other items enabling or disabling opening or closing of liquidity positions must be trivial and easy to do and must not require a new package. | 
| Each supported exchange should have their own liquidity receipt resource with its own unique data. |  |
| Ignition must be resilient to pool price manipulation attacks and ensure that there is a mechanism to detect such price manipulations and deny service when they happen.| | 

## Architecture

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./diagrams/architecture-dark.png">
  <img alt="Text changing depending on mode. Light: 'So light!' Dark: 'So dark!'" src="./diagrams/architecture-light.png">
</picture>

The above image shows the architecture of the Ignition system where there is a core Ignition protocol whose main responsibilities include managing the lockup periods, settlement of funds, the logic around when positions can and can not be opened and closed and other aspects too. The core Ignition protocol component communicates with pools and oracles through adapters which mean that Ignition protocol does not need to concern itself with how the interfaces of the exchanges differ.

With Ignition being a liquidity incentives program a core aspect of it is its ability to communicate with other pools via invocations. This could be to add liquidity to a pool, remove liquidity from a pool, or get the current price of the pair as seen by the pool. Ignition also needs to integrate with the three biggest decentralized exchanges on the Radix network which are Ociswap, Caviarnine, and Defiplaza. However, each exchange has its interface for the aforementioned operations that completely differs from all other exchanges. The table below shows the method signatures for these operations on pools from the various exchanges:

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

A very important requirement for the Ignition system is that the protocol layer that handles the lockup periods, upfront rewards, position settlement, and other protocol aspects be exchange-agnostic and not tied to any particular exchange(s). As such, Ignition uses adapters which are blueprints that are written to translate invocations they receive from Ignition into their equivalent for the exchange that they support and translate returns from the exchange invocations back to the structure expected by Ignition. This means that adapters add a layer of abstraction that makes all pools, from the perspective of the Ignition protocol, have an identical interface and predictable interface. 

An alternative to using adapters would be to code the logic for the various function signatures into the Ignition protocol itself. This loses us the exchange agnosticism and means that the addition of new exchanges down the line requires a completely new Ignition package alongside fund migration to a new component instantiated from said package. Additionally, this violates the [technical requirement](#technical-requirements) _"Ignition must support new exchanges trivially."_.

Another alternative architecture to the above could have the adapters be wrappers where instead of having a single adapter per exchange it would be a single adapter per pool. The largest disadvantage to that approach is that if there is ever a need to update the adapter logic then it would involve updating as many adapters as there are pools, and so the quantity is larger. This violates the [technical requirement](#technical-requirements) _"Ignition must support new exchanges trivially."_.

There is a very clear separation of concerns between the Ignition protocol layer and the adapters:

* Ignition Protocol Layer: It is exchange agnostic and handles the management and enforcement of the lockup period, upfront rewards, position settlement, readiness of the protocol and communication with the adapters.
* Adapters: Provide a unified interface for Ignition to communicate with the exchanges by translating Ignition's invocations into invocations understood by the pools and translating the returns into the structure expected by Ignition. Adapters might also contain logic for fee calculation as all exchange-specific computations must not happen in the Ignition protocol layer.

At the current moment, the Scrypto standard library does not have a good way for developers to define interfaces. The [`scrypto-interface`](./libraries/scrypto-interface/) crate provided in this project is a direct answer to the interface needs of Ignition. The following bulletpoints describe the interface needs of Ignition and how [`scrypto-interface`](./libraries/scrypto-interface/) solves them: 

| Problem | Solution |
| ------- | -------- | 
| Ignition needs a way for the pool and oracle adapter interfaces to be defined. | The [`scrypto-interface`](./libraries/scrypto-interface/) crate solves this through the `define_interface!` macro which allows for the interface to be defined and it then generates a trait and Scrypto, Scrypto Test, and Manifest Builder stubs from the defined interface which provides a simple and type-safe way for calling components that are known to implement this interface. | 
| Adapters need a type-safe way to implement interfaces and avoid regressions or type-mismatches | The [`scrypto-interface`](./libraries/scrypto-interface/) crate solves this the trait generated by the `define_interface!` macro and the `blueprint_with_traits` attribute macro that can be used instead of the `blueprint` macro allowing for traits to be implemented on the blueprint. This trait implementation provides a compile-time check that the blueprint does indeed implement the interface as it is defined. | 

More documentation is available in the [`scrypto-interface`](./libraries/scrypto-interface/) library directory. To summarize what it does: the library allows for interfaces to be defined and for stubs to be generated from the defined interfaces and for compile-time checks that the interfaces are implemented correctly (with matching signatures).

Ignition defines the pool adapter to have the following interface:

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

define_interface! {
    PoolAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            #[manifest_type = "(ManifestBucket, ManifestBucket)"]
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput;
        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            #[manifest_type = "Vec<ManifestBucket>"]
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
```

The above code can be found in full in the [`ports_interface/src/pool.rs`](./libraries/ports-interface/src/pool.rs) and [`ports_interface/src/oracle.rs`](./libraries/ports-interface/src/oracle.rs) files. The code can be interpreted as _"any component that implements the interface defined by `OracleAdapter` can be treated by Ignition as an oracle adapter, and any component that implements the interface `PoolAdapter` can be treated by Ignition as a pool adapter"_. This is quite different from the typical approach seen in the Scrypto standard library which focuses more on the implementation rather than the interface. However, for Ignition to be exchange agnostic it can't tie itself to any particular adapter implementation and has to be able to operate over interfaces.

Each exchange has a single adapter which needs to be registered in Ignition. Ignition's state has a map where the key is the blueprint id (package address and blueprint name) of the exchange pool blueprints and the value is the information about the pools. As such, to support a new exchange in Ignition after launch a new package needs to be authored that implements the adapter interface for that particular exchange and then the adapter and the allowed pools need to be registered on the Ignition component.

With this architecture Ignition features:

* Hot-swappable adapters that can be added, updated, or removed at runtime without the need for a new protocol package.
* The ability to support new exchanges after launch trivially and with ease.
* An upgradable and modular architecture that takes upgradeability into account at every step of the way.
* The ability for oracles and other components to be changed at runtime.

## Testing

Ignition is tested through two classes of tests: integration tests and stateful integration tests. These two classes serve different needs and helped ensure that the final Ignition package could run on the networks we wanted to deploy it on.

### Integration Tests

### Stateful Integration Tests

Most applications can rely on local integration tests alone to ensure that their code behaves as expected. However, in a complex system that contains many different blueprints and components, some of which are developed internally and some of which are developed externally, local integration tests alone are not enough, especially if some of those components and blueprints are not available locally (e.g., Ociswap's production BTC/XRD pool is not available to me locally in resim). For such applications, local integration tests might all pass but a similar test run on a live network might fail for many different reasons. 

This created a need for a way to test Ignition against a live network, real pools, real resources, real exchanges, and real everything, and more specifically against mainnet. However, this also needs to be efficient and inexpensive. We're unwilling to pay actual XRD for each test run or any kind of user resources like BTC or ETH. This creates a contradiction where on one hand we would like to test against a live network with real everything and on the other hand we wish for it to be efficient and to cost nothing.

The solution to that is Ignition's stateful testing framework that allows Ignition to be tested against mainnet state without needing to submit any transactions to the network! It works as follows:

1. The `TestRunner` is able to operate over any substate database so long as it implements some specific traits.
2. A running node has its substate database (part of the state manager) and it is essentially the database that contains the entirety of mainnet state.
3. We can hook up the `TestRunner` to operate over the node's database thus allowing us to have a `TestRunner` with mainnet state!

In addition to the above, the approach that Ignition follows uses a `SubstateDatabaseOverlay` which is a database overlay that the node's database is wrapped in so that any writes to the database are written to the overlay and not to the node's database to avoid the test-runner corrupting the node's database.

This framework allowed for Ignition to be tested against Caviarnine, Ociswap, and Defiplaza pools from mainnet with real resources without costing a penny! The biggest downside is the need for a node to be running, however, this framework could grow to utilize the Core API instead of a locally running node as all that this framework needs is the ability to read state.

The following is an example of a minimal stateful test:

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
