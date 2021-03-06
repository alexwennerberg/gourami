# Admin Guide 

If you want to administer a Gourami server, you'll need a few technical skills:

1. Basic Linux sysadmin skills -- ability to set up a web server.
2. Basic SQL knowledge -- ability to query and insert records. Right now, Gourami does not have an admin interface or admin tools, so certain actions (such as resetting a user's password or deleting a post or account) will require manual SQL intervention. 

## Deployment

Once you've built Gourami, you'll have a standalone binary that runs the server. Nothing else is needed, aside from Sqlite and a TLS cert! If you'd like, you can also set up an Nginx proxy. I put together an ansible playbook that you may find helpful in `ansible/`. You'll need to modify the environment variables (an example is in sample_env) for your production deployment. If you are having trouble deploying Gourami, feel free to send me an email or open a GitHub issue. I plan on writing more detailed instructions for deploying a production server at some point. 

I do not use Docker for my Gourami deployment, so I have not created any Dockerfiles. Since the only dependency not inside the binary is Sqlite, I question the value of using a Docker container.

## Inviting users

Gourami is invite-only. You can control who can invite users by setting the can_invite flag for that user in the user table.

## Connecting with other servers.

Gourami uses ActivityPub to connect with other ActivityPub Actors. If you're familiar with ActivityPub, you should know that Gourami works somewhat differently than a service like Mastodon.

Gourami connects through the "neighborhood" timeline. This means that any post that a user on your server makes in the neighborhood timeline is sent to all servers you are connected with. You can connect with either a server or an individual ActivityPub actor, such as a Mastodon user, but be aware that that user will see all posts in your neighborhood timeline.  You will only be considered "connected" to a remote server if you follow that server and that server follows you back.

Gourami doesn't implement unfollows yet, so you'll have to directly modify the database and communicate with the user / server you're unfollowing.

## Social guidelines

Gourami is built for small deployments -- I have not tested it or designed it for larger implementations. This gives users a lot more flexibility, but requires more trust on your end. For example, a user may be easily able to spam the timeline, spam everyone's notifications, DOS the server, etc, so as an admin you should only allow people on your instance that you trust. You will also find that the quality of the shared timelines will begin to degrade after too many users. If you're still interested in attempting a larger Gourami deployment, I won't stop you, but beware that you're in uncharted territory.

## Customizing Gourami

You may want to customize parts of Gourami, such as the CSS format or server message. Right now, html templates are compiled into the binary. In retrospect, it might have been a better idea to use a templating engine that is rendered at runtime. If you want to customize the html, you'll have to edit the file and recompile. I may move towards a different templating library at some point.

## Gourami's ActivityPub implementation

Gourami's ActivityPub implementation is somewhat opinionated and a little esoteric. Gourami is not Mastodon or Twitter and is not trying to be. Using it in that way may cause some frustration -- so just be aware that Gourami does things a little differently. I'm considering adding more 'traditional' activitypub functionality.

The server has a server actor. This is an ActivityPub actor of type "Organization" and is the only ActivityPub actor on the server. All requests go through this actor. This forces you to think of your server as a cohesive whole -- users or other servers can only follow an entire server, not individual users. I encourage you to think about how this would change the way you structure your community. The distinction between which user says something is done via a string at the beginning of the post content. Gourami will parse this string as the user, other services will not.

Currently, deletes are not supported. 

The only audience currently supported for ingoing and outgoing messages is Public.
