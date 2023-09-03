# ProtoT protobuf schema

The core communication protocols and other util objects that are commonly in use at ProtoT core library, are implemented and structured in `Protocol Buffers (Protobuf)`.

We are generating the protobuf code with `Sylk Build CLI`, a protobuf developer toolchain for easy setup and unification in the way we structuring our `ProtoT` schema.

> Disclaimer we are also the authors of `Sylk Build CLI`.

## Packages

- `sylklabs` - The main "domain" of our project (root namespace)
    - `core` - Commonly shared objects that are not subject to versioning at this momenet (We are considring them as long lasting schema, as they serve abstractions that will not need breaking changes in the near future)
    - `scheduler` - The main "Buisness logic" protocols for communication with the `ProtoT Scheduler Server` and `Worker`s nodes.
        - `v1` - The current version

In short we have the following logical paths:

```sh
sylklabs.core
sylklabs.scheduler.v1
```

## Important Entities

ProtoT is based on some abstraction that is made to the general idea of "Task", and its functionality with a hunmbled view that we want to MAYABE (most certinely will) extend our abstraction and capabilities with core types and other "pluggable" types.

Here are some of the most important "Building blocks" of ProtoT schema:

- `sylklabs.core.Task` - Our main entity for the abstraction of "Task"
- `sylklabs.core.Config` - Some commonly used configuration structure for ProtoT runtime values
- `sylklabs.scheduler.v1.SchedulerService` - The main API fot ProtoT scheduler server

## Compile

To evolve the schema we mentioned earlier we are using `Sylk Build CLI`, but for compilation we can use any of the available methods:

- __Rust users:__ User that going to work with native rust is best to use `prost` and espacially the sub package `prost-build` which easily integrate with existing rust projects.
- __Non rust users:__ We recommend to use `Sylk Build CLI` but you can compile your prorotbuf with native `protoc` usage, as long it follow the fllowing rules:
    * This repo is for Rust impl. only and should stay without any boilerplate client / server side impl. in other languages, so the generated code you __MAY__ want to add for your own good is fine as long it dosent turn out to be released in the next version of `ProtoT` alongside the rust __ONLY__ impl.

## Linting rules

- MUST Use `snake_case` for fields, enum values.
- MUST Use `TitleCase` for methods (RPC's), message, enums, services.
- MUST Use `XxxService` suffix for all services.
- MUST Use `SHOUTING_CASE` for enum values.
- MAY Use `PREFIX_XXX` for enum values.
- MAY Use `some.package.v1` version  components in package paths, if they are presented in package path MUST use them as the last path component, e.g each package full path that use version component in it's logical name must end with a version component.
- `.proto` file MUST declare a logical package and the path MUST be exact mirror of file system hierarchy of the protobuf files, also file names MUST use `snake_case`.
