# Statefun Rust SDK

An SDK for writing stateful functions in Rust. See the [Apache Flink Stateful
Functions](https://flink.apache.org/stateful-functions.html) website for more
information about the project.

## Supported StateFun API version

This library currently targets [Statefun v3.2.0](https://flink.apache.org/2022/01/31/stateful-functions-3.2.0-release-announcement/).

## Changes since v3.x of Apache Statefun

Please note that Apache Statefun v3.x has API breaking changes. This version of the Rust SDK
only supports v3.x of the Apache Statefun protocol.

## How to use

There are a few examples provided in the `./examples` directory. Please refer to each example's
readme file fore more info.

Please also refer to the [documentation](https://docs.rs/statefun) to learn more on how to
set up & use the Statefun cluster.

## Building

You need to have the Protobuf compiler `protoc` available in your `$PATH`. On
macOS you can install this via

```
brew install protobuf
```
