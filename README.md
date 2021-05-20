# SLISKO

**Installation**

To run on Linux, install the following packages: libgl1-mesa-dev xorg-dev libglfw-dev

**Architecture**
```
                        ┌───────────────────────────────────┐                                                   
                        │        ┌──────────────────────────┼───┐   ┌───────────────────────────────┐           
                        │        │ Simulator                │   │   │ APA102                        │           
                        │        │                         ┌─┐  │   │                         ┌─┐   │           
                        │        │  ch *Chassi             │ │  │   │  rs chan RenderSignal   │ │◀┐ │           
                        │        │                         └─┘  │   │                         └─┘ │ │           
                        │        │                         ┌─┐  │   │                         ┌─┐ │ │           
                        │        │  rs chan RenderSignal   │ │  │   │  mapping []*pixel.Pixel │ │─┼─┼──────────┐
                        │        │                         └─┘  │   │                         └─┘ │ │          │
                        │        └──────────────────────────▲───┘   └─────────────────────────────┼─┘          │
                        │                                   │                                     │            │
                        │                                   │                                     │            │
                        │ ┌──────────────────┐              └─────────────────────────────────────┤            │
                        │ │                  │                                                    │            │
                        │ │                  ▼                                                    │            │
                        │ │      ┌───────────────────────┐    ┌───────────────────────────────────┼─┐          │
                        │ │      │Controller             │    │Broker                             │ │          │
                        │ │      │                    ┌─┐│    │                                  ┌─┐│          │
                        │ │      │ FrameBroker *Broker│ │├───▶│ Subscribe() -> chan RenderSignal │ ││          │
                        │ │      │                    └─┘│    │ Unsubscribe()                    └─┘│          │
                        │ │      │ EnablePattern(str)    │    │ Publish()                           │          │
                        │ │      │ DisablePattern(str)   │    └─────────────────────────────────────┘          │
                        │ │      │ ┌─────────────┐       │                                                     │
                        │ │   ┌──┼─│   *Chassi   │       │                                                     │
                        │ │   │  │ └─────────────┘       │                                                     │
                        │ │   │  │ ┌─────────────┐       │    ┌──────────────────┐                             │
                        │ │   │  │ │[]Patterns   │       │    │Pattern           │                             │
                        │ │   │  │ │┌─┐┌─┐┌─┐┌─┐ │       │    │                  │                             │
                        │ │   │  │ ││ ││ ││ ││ │─┼───────┼───▶│ Render(*Chassi)  │                             │
                        │ │   │  │ │└─┘└─┘└─┘└─┘ │       │    │                  │                             │
                        │ │   │  │ └─────────────┘       │    │ Info()           │                             │
                        │ │   │  └───────────────────────┘    └──────────────────┘                             │
                        │ │   │  ┌───────────────────────────────────────────────────────────────────────────┐ │
                        │ │   │  │Chassi                                                                     │ │
                        │ │   │  │┌──────────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐ │ │
┌────────────────────┐  │ │   │  ││ Linecard             │ │ Linecard             │ │ Linecard             │ │ │
│API                 │  │ │   │  ││                      │ │                      │ │                      │ │ │
│                 ┌─┐│  │ │   │  ││  Name string         │ │  Name string         │ │  Name string         │ │ │
│ ctrl *Controller│ │├──┼─┘   │  ││  Image string        │ │  Image string        │ │  Image string        │ │ │
│                 ├─┤│  │     │  ││  Active boolk        │ │  Active boolk        │ │  Active boolk        │ │ │
│ ch *Chassi      │ │├──┼───┬─┴─▶││  LEDs ┌──────────┐   │ │  LEDs ┌─────────────┐│ │  LEDs ┌─────────────┐│ │ │
│                 └─┘│  │   │    ││       │[]Pixel   │   │ │       │[]Pixel      ││ │       │[]Pixel      ││ │ │
└────────────────────┘  │   │    ││       │┌─┐┌─┐┌─┐ │   │ │       │┌─┐┌─┐┌─┐┌─┐ ││ │       │┌─┐┌─┐┌─┐┌─┐ ││ │ │
                        │   │    ││       ││ ││ ││ │─┼─┐ │ │       ││ ││ ││ ││ │ ││ │       ││ ││ ││ ││ │ ││ │ │
                        └───┘    ││       │└─┘└─┘└─┘ │ │ │ │       │└─┘└─┘└─┘└─┘ ││ │       │└─┘└─┘└─┘└─┘ ││ │ │
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