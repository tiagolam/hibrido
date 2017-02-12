# Hibrido

Hibrido is an implementation of a WebRTC gateway using the Rust language.

###### Disclaimer

At this point this is mainly a PoC project that's being developed at the same
time some concepts are being grasped, around Rust and WebRTC.
 
For this initial PoC, only the SFU model is being considered, although the
roadmap is a little more ambitious.

## Overview

State: PoC

The name "hibrido" means hybrid in Portuguese, and this is because in its
final form, this project intends to provide an implementation of both models
of multiparty videoconferencing in WebRTC, the SFU vs CMU topologies, and
make it possible to choose appropriately between one of them when creating a
conference (convo).

## Requirements

In order to build and use this PoC one needs to clone the [rir
lib](https://github.com/tiagolam/rir) (an RTP implementation in Rust) and
link to it in this project's dependencies - this is only a temporary
requirement while there isn't a crate for that lib.

## Architecture

Since this is still just a PoC, and there's a lot of Rust learning going on
at the moment, the architecture is still being defined.

