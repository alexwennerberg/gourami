# Gourami

An intentionally small, ultra-lightweight social media network (ActivityPub integration TBD)

## Philosophy and Design

Gourami differs from existing social networks in a number of ways:
* Intentionally small -- designed to support 50-100 active users. I'm sure it could support more, but things could quickly become a mess.
* Invite-only and closed -- a community curated by the server admin, rather than open to all.
* Extremely lightweight & fast -- very little Javascript, plain text, small page sizes. Should run on extremely lightweight/cheap hardware and low-bandwidth networks. 
* [Brutalist](https://brutalist-web.design/) -- Stark and minimal, the design and interface should emphasize, rather than hide, the underlying building blocks of the web that comprise it. This will give Gourami a feel similar to 90s or 2000s web forums. 
* Simple and feature-averse -- A simpler Gourami is much easier for me to develop, support and maintain. I want Gourami to be reliable software that people can build communities on top of, and severely limiting the feature set makes that much easier.

Some goals of this project that are work in progress:
* Support for [ActivityPub](https://activitypub.rocks/) federation
* Authentication-free mode: For deployment on private networks, such as on a local wireless network.
* Additional accessibility features

Read [this document](https://git.sr.ht/~alexwennerberg/gourami-social/tree/master/PHILOSOPHY.md) for more

## Dependencies:

* sqlite3
* sqlite3-dev

## Installation

[Install Rustup](https://doc.rust-lang.org/cargo/getting-started/installation.html) or Cargo

Clone this repository.

Run `cargo install --path .`

Set environment variables (todo -- add more details)

Run the local server with `gourami_social run`

## Deployment

Get a Linux box and configure it. Set up ssl, firewall rules, logging, etc.

I think it'd be interesting to set this up so that it can be deployed on a Platform as a Service or Function as a Service offering, but I haven't found any great way to run Sqlite in that context. I may put together an ansible playbook or something.
