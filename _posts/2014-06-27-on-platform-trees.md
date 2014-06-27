---
layout: post
title:  Platform trees in zinc
categories: platformtree
---

Welcome to the zinc.rs blog, where I'll post my discoveries on the path of using [rust][rust] for embedded development. For today's topic I'll share my vision on platform trees.

While I've been prototyping some code for network stack, I figured that there's no nice way to define IRQ callback functions in a way that would be compact, effective and safe. Rust is all about safety so I really do my best in terms of providing safe API to hardware. I went to [device trees][dt] for some inspiration and figured that I can provide a safe way to initialize and manage hardware with some metaprogramming. And rust is extremely effective for metaprogramming allowing one to take a stream of tokens and produce some AST. While at that, I decided to hide all the platform-specific initializations and RTOS hacks, and that's how platform tree was born.

Platform tree is a structure that contains nodes where each node can have some child nodes and/or attributes. It's effectively XML without the namespaces and more human-friendly syntax. Apart from a few specific cases, nodes, child nodes and attributes are excessively checked for any inconsistencies. PT parser would not allow one to define PLL configuration that would burn the target MCU. All the verification checks are done in same rust code but at compile time. In the end, PT compiles down to `hal::mcu::whatever` calls, so there's nothing you can do with PT that you cannot do without it.

[rust]: http://www.rust-lang.org
[dt]:   http://en.wikipedia.org/wiki/Device_tree
