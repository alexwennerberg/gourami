# 🐟gourami
![Build and Test](https://github.com/alexwennerberg/gourami/workflows/Build%20and%20Test/badge.svg)

An intentionally small, ultra-lightweight ActivityPub social network. See a live public demo at https://dev.gourami.social/ and reach out to me if you want an invite so you can try it out.

![image](docs/demo.jpg)

## Philosophy and Design

Gourami differs from existing social networks in a number of ways:
* **Intentionally small** -- designed to support 50-100 active users. I'm sure it could support more, but things could quickly become a mess. Gourami was hugely and directly inspired by the fantastic essay on [runyourown.social](https://runyourown.social)
* **Invite-only and closed** -- a community curated by the server admin, rather than open to all.
* **Free and open source** -- I find the privatization of the internet extremely concerning, especially the way that the very space for building community and networking with our friends is controlled by for-profit corporations with potentially different values and goals than their users.
* **Decentralized** -- Gourami uses [ActivityPub](https://activitypub.rocks/) for federation, but with an implementation that differs from existing ActivityPub servers such as Mastodon. Instances federate at the server level, rather than the user level, which means all users on the server share the same "neighborhood".
* **A social network with physical context** -- Gourami should be easy to deploy in a physical space (such as a coffee shop or a local wireless network) or among people in a specific physical community, such as a school. In *How to Do Nothing*, Jenny Odell discusses the lack of a context, specifically physical and temporal context, in social media, and, while praising Mastodon, also calls for social networks that are tied to physical space. While Gourami does not force you to tie a deployment to a place, it is designed in such a way that such a deployment would be relatively easy.
* **Extremely lightweight & fast** -- very little Javascript, plain text, small page sizes. Should run on extremely lightweight/cheap hardware and low-bandwidth networks. 
* **[Brutalist](https://brutalist-web.design/)** -- Stark and minimal, the design and interface should emphasize, rather than hide, the underlying building blocks of the web that comprise it. This will give Gourami a feel similar to 90s or 2000s web forums. 
* **Simple and feature-averse** -- A simpler Gourami is much easier for me to develop, support and maintain. I want Gourami to be reliable software that people can build communities on top of, and severely limiting the feature set makes that much easier. Once I get Gourami to a certain core feature set, my work will be dedicated to maintenance and care, rather than feature additions. This will allow people to develop long-term, stable social networks, and also develop forks without worrying about losing upstream changes.

## Dependencies:

* sqlite3
* libsqlite3-dev
* openssl

## Installation

I put together a demo container! -- Link

[Install Rustup](https://doc.rust-lang.org/cargo/getting-started/installation.html) or Cargo

Clone this repository.

Run `cargo install --path .`

Set environment variables (todo -- add more details)

Configuration is done via [dotenv](https://github.com/dotenv-rs/dotenv)

Run the local server with `gourami_social run`

Setup the database with  

## Deployment

**Gourami is in alpha / development stage.**

I would love if you gave Gourami a shot, but be aware that issues may arise. If you would like to follow or join my server, please reach out to me! I'm also happy to help anyone set up their instance.

## References

Many projects inspired my work here, and I want to mention them

* Jenny Odell's discussion of adding context to social media in *How to Do Nothing*
* https://runyourown.social/
* https://joinmastodon.org/
* https://sourcehut.org/    
* https://100r.co/site/mission.html
* https://solar.lowtechmagazine.com/2020/01/how-sustainable-is-a-solar-powered-website.html
* https://www.nycmesh.net/
* [Honk](https://flak.tedunangst.com/post/ActivityPub-as-it-has-been-understood) and Ted Unangst's work with ActivityPub
* https://github.com/rustodon/rustodon
* https://github.com/LemmyNet/lemmy

logo from https://twemoji.twitter.com/
