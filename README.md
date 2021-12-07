# Arcswap

Uniswap-inspired automated market-maker (AMM) protocol powered by Smart Contracts on the Archway blockchain.

This contract allows you to swap native cosmos coins for cw20 tokens. Liquidity providers can add liquidity to the market and receive a 0.03% fee on every transaction.

# Deployments

- uco token: `archway14nurau9scuqhr3sczktx63sr8kdpvwh0zynv63`
- uco/uconst pair pool contract: `archway1upvys78p4whku20c0addqurdt65y3l02ufh8wa`

# Usage

The following instructions are written for the constantine-1 testnet.

## Deploy
```
archwayd tx wasm instantiate 3 '{"native_denom": "<native_denom>", "token_address":"<cw20_contract_address>", "token_denom": "<token_denom>"}' --from <key>  --chain-id="constantine-1"
```

## Swap Native for token

This swaps the native token for the cw20 token. Use the price query below to estimate the price before executing this message. min_token is used to set limit on acceptable price for the swap.
```
archwayd tx wasm execute archway1unyuj8qnmygvzuex3dwmg9yzt9alhvye2canzt '{"swap_native_for_token_to":{"min_token":"0", "recipient": "archway10zq3c4zk8fhc6yzf9netgdcwfu0q2777gtlx6q"}}'  --amount "2uconst" --from Coco --node "https://rpc.constantine-1.archway.tech:443" --chain-id constantine-1
```

## Add Liquidity
This message adds liquidity to the pool and give the caller proportional ownership of pool funds. Funds need to be deposited at the current ratio of the pools reserves, ie if the pool currently has 100 native tokens and 300 cw20 tokens the caller needs to deposit at a ratio of 1 to 3. Max token should be set a little higher than expected in case there are any changes in the pool reserves.
```
archwayd tx wasm execute archway1zwv6feuzhy6a9wekh96cd57lsarmqlwxn054te '{"increase_allowance":{"amount":"1000000","spender":"archway1unyuj8qnmygvzuex3dwmg9yzt9alhvye2canzt"}}' --from Coco --node "https://rpc.constantine-1.archway.tech:443" --chain-id constantine-1
```
```
archwayd tx wasm execute archway1unyuj8qnmygvzuex3dwmg9yzt9alhvye2canzt '{"add_liquidity":{"max_token":"1000","min_liquidity":"0"}}'  --amount "100uconst" --from Coco --node "https://rpc.constantine-1.archway.tech:443/" --chain-id constantine-1
```

## Withdraw Liquidity
This removes liquidity from the pool and returns it to the owner. Current liquidity owner ship can be seen with the balance query below. min_native and min_token are used to ensure the pool reserves do no unexpectedly change. Set both values to 1 if you want to guarantee the message is executed.
```
archwayd tx wasm execute archway1unyuj8qnmygvzuex3dwmg9yzt9alhvye2canzt '{"remove_liquidity":{"amount": "2", "min_native":"1","min_token":"1"}}' --from Coco --node "https://rpc.constantine-1.archway.tech:443/" --chain-id constantine-1
```

## Creating a new repo from template

Assuming you have a recent version of rust and cargo (v1.51.0+) installed
(via [rustup](https://rustup.rs/)),
then the following should get you a new repo to start a contract:


Install [cargo-generate](https://github.com/ashleygwilliams/cargo-generate) and cargo-run-script.
Unless you did that before, run this line now:

```sh
cargo install cargo-generate cargo-run-script --features vendored-openssl 
```

Now, use it to create your new contract.
Go to the folder in which you want to place it and run:


**Latest: 0.16**

```sh
cargo generate --git git@github.com:drewstaylor/archway-template.git --name PROJECT_NAME
````

**Older Version**

Pass version as branch flag:

```sh
cargo generate --git git@github.com:drewstaylor/archway-template.git --branch <version> --name PROJECT_NAME
````

Example:

```sh
cargo generate --git git@github.com:drewstaylor/archway-template.git --branch 0.14 --name PROJECT_NAME
```

You will now have a new folder called `PROJECT_NAME` (I hope you changed that to something else)
containing a simple working contract and build system that you can customize.

## Create a Repo

After generating, you have a initialized local git repo, but no commits, and no remote.
Go to a server (eg. github) and create a new upstream repo (called `YOUR-GIT-URL` below).
Then run the following:

```sh
# this is needed to create a valid Cargo.lock file (see below)
cargo check
git branch -M main
git add .
git commit -m 'Initial Commit'
git remote add origin YOUR-GIT-URL
git push -u origin master
```

## CI Support

We have template configurations for both [GitHub Actions](.github/workflows/Basic.yml)
and [Circle CI](.circleci/config.yml) in the generated project, so you can
get up and running with CI right away.

One note is that the CI runs all `cargo` commands
with `--locked` to ensure it uses the exact same versions as you have locally. This also means
you must have an up-to-date `Cargo.lock` file, which is not auto-generated.
The first time you set up the project (or after adding any dep), you should ensure the
`Cargo.lock` file is updated, so the CI will test properly. This can be done simply by
running `cargo check` or `cargo unit-test`.

## Using your project

Once you have your custom repo, you should check out [Developing](./Developing.md) to explain
more on how to run tests and develop code. Or go through the
[online tutorial](https://docs.archway.io/docs/create/guides/my-first-dapp/start) to get a better feel
of how to develop.

[Publishing](./Publishing.md) contains useful information on how to publish your contract
to the world, once you are ready to deploy it on a running blockchain. And
[Importing](./Importing.md) contains information about pulling in other contracts or crates
that have been published.

Please replace this README file with information about your specific project. You can keep
the `Developing.md` and `Publishing.md` files as useful referenced, but please set some
proper description in the README.

## Gitpod integration

[Gitpod](https://www.gitpod.io/) container-based development platform will be enabled on your project by default.

Workspace contains:
 - **rust**: for builds
 - [wasmd](https://github.com/CosmWasm/wasmd): for local node setup and client
 - **jq**: shell JSON manipulation tool

Follow [Gitpod Getting Started](https://www.gitpod.io/docs/getting-started) and launch your workspace.

