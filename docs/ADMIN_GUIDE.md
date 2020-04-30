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

## Securing your server

I would recommend following basic Linux syadmin best practices: disable password login, consider a hardened Linux distro, set up a firewall, etc. I'm not a security expert here, I would recommend following guides produced by those who are.

## Gourami's ActivityPub implementation

(maybe this should be somewhere else -- like a dev guide)

## Federation -- the "neighborhood"

## Passwordless local deployment

Don't do this on the public internet, it is a bad idea and will only lead to ruin! Seriously, don't do it.
