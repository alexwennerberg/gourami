(WIP -- it doesnt actually work like this yet)

# User Guide

You will get a notification when someone tags your username in a post or if they reply to your post.

## Creating a note

Enter your text in the note box, then click create note. The first post that you tag (via 📝X or >>X, where X is the post number) will be considered a post(s) you're replying to.  

For every local post or user that you tag in a note (using @), that user will receive a notification.

HTML tags will be stripped from your note input, except for p, br, and span.

(write about how remote notes are handled)

## Audiences

Whenever a user creates a note, that note is visible to all the users on this server, which can be seen on the "local" timeline".

Gourami has a feature called the "neighbhorhood", which allows one Gourami server to communicate with other social media services. 

The Neighbhorhood timeline consists of all other [ActivityPub](http://activitypub.rocks/) services. If you're not familiar with ActivityPub, it's a shared language that allows different social media applications to communicate with each other. This means that two services that both implement ActivityPub (such as Gourami and [Mastodon](https://joinmastodon.org/) should be able to communicate with each other. In practice, there may be differences between each individual ActivityPub services, 

A note on permissions --

While local-only posts are private, a neighborhood post WILL be sent to other servers. Only add servers to your neighborhood that you trust! 

todo -- explain more
