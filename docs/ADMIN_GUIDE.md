(WIP -- it doesnt actually work like this yet)

# Admin Guide 

If you want to administer a Gourami server, you'll need a few technical skills:

1. Basic Linux sysadmin skills -- ability to set up a webserver.
2. Basic SQL knowledge -- ability to query and insert records. Right now, Gourami does not have an admin interface or admin tools, so certain actions (such as resetting a user's password or deleting a post or account) will require manual SQL intervention. 

## Inviting users

Gourami is invite-only. Right now, you create an invite by adding a record to the invitation_keys table and sharing that key with the user you're inviting.

## Social guidelines

Gourami is built for small deployments -- I have not tested it or designed it for larger implementations. This gives users a lot more flexibility, but requires more trust on your end. For example, a user may be easily able to spam the timeline, spam everyone's notifications, DOS the server, etc, so as an admin you should only allow people on your instance that you trust. You will also find that the quality of the shared timelines will begin to degrade after too many users. If you're still interested in attempting a larger Gourami deployment, I won't stop you, but beware that you're in uncharted territory.

I'm not big into formal rules or codes of context, but if you feel like that's important for your server, you may want to put it in your server message.

## Customizing Gourami

You may want to customize parts of Gourami, such as the CSS format or server message. Right now, html templates are compiled into the binary. In retrospect, it might have been a better idea to use a templating engine that is rendered at runtime. If you want to customize the html, you'll have to edit the file and recompile. I may move towards a different templating library at some point.


## Securing your server

I would recommend following basic Linux syadmin best practices: disable password login, consider a hardened Linux distro, set up a firewall, etc. I'm not a security expert here, I would recommend following guides produced by those who are.

## Gourami's ActivityPub implementation

Gourami's ActivityPub implementation is somewhat opinionated and a little esoteric. Gourami is not Mastodon or Twitter and is not trying to be. Using it in that way may cause some frustration -- so just be aware that Gourami does things a little differently. I'm considering adding more 'traditional' activitypub functionality.

The server has a server actor. This is an ActivityPub actor of type "Organization" and is the only ActivityPub actor on the server. All requests go through this actor. This forces you to think of your server as a cohesive whole -- users or other servers can only follow an entire server, not individual users. I encourage you to think about how this would change the way you structure your community.

Currently, deletes are not supported. Deletes can be misleading in federation, and I think the simplest solution is just not to implement them.

The only audience supported for ingoing and outgoing messages is [public]. This both simplifies the AP implementation and, in my view, more accurately specifies how ActivityPub works in practice -- once I send my message to a remote server, there isn't really any guarantee as to where it will go.

Most of these decisions were informed by simplicity

## Federation -- the "neighborhood"

Use the admin command follow to follow a server.
That server must accept your follow, then follow you back in order to be in the neighborhood. There is no one-way following in gourami, we (ab)use the AP standard to force mutual follows

## Passwordless local deployment

Don't do this on the public internet, it is a bad idea and will only lead to ruin! Seriously, don't do it.

## Federation

ActivityPub varies across servers. Some functionality may not work with other AP servers. Examples of things that may break include:

* HTML tags that aren't supported getting sanitized
* A different key algorithm being used for HTML signatures
* Custom service-specific activitypub features
* AP features supported by their server but not Gourami (Gourami is extremely limited in its interpretation of ActivityPub)
