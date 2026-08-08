# SLISKO

> This is the preserved legacy Go implementation. The Rust implementation is
> now the repository root. Shared images and TOML configurations remain in
> `../assets` and `../configurations`.

Run the legacy application from this directory:

```sh
go test ./...
go run . --simulator
```

The Vue distribution remains in `ui/dist` and is served by the Go API. The
Raspberry Pi installer is invoked from the repository root as
`sudo ./prev/rpi/install.sh`.

**Installation**

To run on Linux, install the following packages: libgl1-mesa-dev xorg-dev build-essential libglfw3-dev

**Architecture**
```
┌───────────────────────────────┐                                                   
│    ┌──────────────────────────┼───┐   ┌───────────────────────────────┐           
│    │ Simulator                │   │   │ APA102                        │           
│    │                         ┌─┐  │   │                         ┌─┐   │           
│    │  ch *Chassi             │ │  │   │  rs chan RenderSignal   │ │◀┐ │           
│    │                         └─┘  │   │                         └─┘ │ │           
│    │                         ┌─┐  │   │                         ┌─┐ │ │           
│    │  rs chan RenderSignal   │ │  │   │  mapping []*pixel.Pixel │ │─┼─┼──────────┐
│    │                         └─┘  │   │                         └─┘ │ │          │
│    └──────────────────────────▲───┘   └─────────────────────────────┼─┘          │
│                               │                                     │            │
│                               │                                     │            │
│                               └─────────────────────────────────────┤            │
│                                                                     │            │
│                                                                     │            │
│    ┌───────────────────────┐    ┌───────────────────────────────────┼─┐          │
│    │Controller             │    │Broker                             │ │          │
│    │                    ┌─┐│    │                                  ┌─┐│          │
│    │ FrameBroker *Broker│ │├───▶│ Subscribe() -> chan RenderSignal │ ││          │
│    │                    └─┘│    │ Unsubscribe()                    └─┘│          │
│    │ EnablePattern(str)    │    │ Publish()                           │          │
│    │ DisablePattern(str)   │    └─────────────────────────────────────┘          │
│    │ ┌─────────────┐       │                                                     │
│ ┌──┼─│   *Chassi   │       │                             ┌────────────────────┐  │
│ │  │ └─────────────┘       │                             │API                 │  │
│ │  │ ┌─────────────┐       │    ┌──────────────────┐     │                 ┌─┐│  │
│ │  │ │[]Patterns   │       │    │Pattern           │     │ ctrl *Controller│ ││  │
│ │  │ │┌─┐┌─┐┌─┐┌─┐ │       │    │                  │     │                 ├─┤│  │
│ │  │ ││ ││ ││ ││ │─┼───────┼───▶│ Render(*Chassi)  │     │ ch *Chassi      │ ││  │
│ │  │ │└─┘└─┘└─┘└─┘ │       │    │                  │     │                 └─┘│  │
│ │  │ └─────────────┘       │    │ Info()           │     └────────────────────┘  │
│ │  └───────────────────────┘    └──────────────────┘                             │
│ │  ┌───────────────────────────────────────────────────────────────────────────┐ │
│ │  │Chassi                                                                     │ │
│ │  │┌──────────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐ │ │
│ │  ││ Linecard             │ │ Linecard             │ │ Linecard             │ │ │
│ │  ││                      │ │                      │ │                      │ │ │
│ │  ││  Name string         │ │  Name string         │ │  Name string         │ │ │
│ │  ││  Image string        │ │  Image string        │ │  Image string        │ │ │
│ │  ││  Active boolk        │ │  Active boolk        │ │  Active boolk        │ │ │
└─┴─▶││  LEDs ┌──────────┐   │ │  LEDs ┌─────────────┐│ │  LEDs ┌─────────────┐│ │ │
     ││       │[]Pixel   │   │ │       │[]Pixel      ││ │       │[]Pixel      ││ │ │
     ││       │┌─┐┌─┐┌─┐ │   │ │       │┌─┐┌─┐┌─┐┌─┐ ││ │       │┌─┐┌─┐┌─┐┌─┐ ││ │ │
     ││       ││ ││ ││ │─┼─┐ │ │       ││ ││ ││ ││ │ ││ │       ││ ││ ││ ││ │ ││ │ │
     ││       │└─┘└─┘└─┘ │ │ │ │       │└─┘└─┘└─┘└─┘ ││ │       │└─┘└─┘└─┘└─┘ ││ │ │
     ││       └──────────┘ │ │ │       └─────────────┘│ │       └─────────────┘│ │ │
     │└────────────────────┼─┘ └──────────────────────┘ └──────────────────────┘ │ │
     └─────────────────────┼─────────────────────────────────────────────────────┘ │
                           │                                                       │
                           │    ┌─────────────────────┐                            │
                           │    │Pixel                │                            │
                           │    │                     │                            │
                           │    │    R f64            │                            │
                           └───▶│    G f64            │◀───────────────────────────┘
                                │    B f64            │                             
                                │    pos (X, Y, Size) │                             
                                └─────────────────────┘                             
```
