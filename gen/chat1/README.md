<!---
DO NOT EDIT !
This file was generated automatically from 'src/generator/templates/api/README.md.mako'
DO NOT EDIT !
-->
The `google-chat1` library allows access to all features of the *Google Hangouts Chat* service.

This documentation was generated from *Hangouts Chat* crate version *7.0.0+20251214*, where *20251214* is the exact revision of the *chat:v1* schema built by the [mako](http://www.makotemplates.org/) code generator *v7.0.0*.

Everything else about the *Hangouts Chat* *v1* API can be found at the
[official documentation site](https://developers.google.com/workspace/chat).
# Features

Handle the following *Resources* with ease from the central [hub](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/HangoutsChat) ...

* [custom emojis](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::CustomEmoji)
 * [*create*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::CustomEmojiCreateCall), [*delete*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::CustomEmojiDeleteCall), [*get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::CustomEmojiGetCall) and [*list*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::CustomEmojiListCall)
* [media](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::Media)
 * [*download*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::MediaDownloadCall) and [*upload*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::MediaUploadCall)
* [spaces](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::Space)
 * [*complete import*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceCompleteImportCall), [*create*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceCreateCall), [*delete*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceDeleteCall), [*find direct message*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceFindDirectMessageCall), [*get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceGetCall), [*list*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceListCall), [*members create*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMemberCreateCall), [*members delete*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMemberDeleteCall), [*members get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMemberGetCall), [*members list*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMemberListCall), [*members patch*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMemberPatchCall), [*messages attachments get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageAttachmentGetCall), [*messages create*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageCreateCall), [*messages delete*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageDeleteCall), [*messages get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageGetCall), [*messages list*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageListCall), [*messages patch*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessagePatchCall), [*messages reactions create*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageReactionCreateCall), [*messages reactions delete*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageReactionDeleteCall), [*messages reactions list*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageReactionListCall), [*messages update*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceMessageUpdateCall), [*patch*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpacePatchCall), [*search*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceSearchCall), [*setup*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceSetupCall), [*space events get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceSpaceEventGetCall) and [*space events list*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::SpaceSpaceEventListCall)
* [users](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::User)
 * [*spaces get space read state*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::UserSpaceGetSpaceReadStateCall), [*spaces space notification setting get*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::UserSpaceSpaceNotificationSettingGetCall), [*spaces space notification setting patch*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::UserSpaceSpaceNotificationSettingPatchCall), [*spaces threads get thread read state*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::UserSpaceThreadGetThreadReadStateCall) and [*spaces update space read state*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::UserSpaceUpdateSpaceReadStateCall)


Upload supported by ...

* [*upload media*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::MediaUploadCall)

Download supported by ...

* [*download media*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/api::MediaDownloadCall)



# Structure of this Library

The API is structured into the following primary items:

* **[Hub](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/HangoutsChat)**
    * a central object to maintain state and allow accessing all *Activities*
    * creates [*Method Builders*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::MethodsBuilder) which in turn
      allow access to individual [*Call Builders*](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::CallBuilder)
* **[Resources](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Resource)**
    * primary types that you can apply *Activities* to
    * a collection of properties and *Parts*
    * **[Parts](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Part)**
        * a collection of properties
        * never directly used in *Activities*
* **[Activities](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::CallBuilder)**
    * operations to apply to *Resources*

All *structures* are marked with applicable traits to further categorize them and ease browsing.

Generally speaking, you can invoke *Activities* like this:

```Rust,ignore
let r = hub.resource().activity(...).doit().await
```

Or specifically ...

```ignore
let r = hub.spaces().members_create(...).doit().await
let r = hub.spaces().members_delete(...).doit().await
let r = hub.spaces().members_get(...).doit().await
let r = hub.spaces().members_list(...).doit().await
let r = hub.spaces().members_patch(...).doit().await
let r = hub.spaces().messages_attachments_get(...).doit().await
let r = hub.spaces().messages_reactions_create(...).doit().await
let r = hub.spaces().messages_reactions_delete(...).doit().await
let r = hub.spaces().messages_reactions_list(...).doit().await
let r = hub.spaces().messages_create(...).doit().await
let r = hub.spaces().messages_delete(...).doit().await
let r = hub.spaces().messages_get(...).doit().await
let r = hub.spaces().messages_list(...).doit().await
let r = hub.spaces().messages_patch(...).doit().await
let r = hub.spaces().messages_update(...).doit().await
let r = hub.spaces().space_events_get(...).doit().await
let r = hub.spaces().space_events_list(...).doit().await
let r = hub.spaces().complete_import(...).doit().await
let r = hub.spaces().create(...).doit().await
let r = hub.spaces().delete(...).doit().await
let r = hub.spaces().find_direct_message(...).doit().await
let r = hub.spaces().get(...).doit().await
let r = hub.spaces().list(...).doit().await
let r = hub.spaces().patch(...).doit().await
let r = hub.spaces().search(...).doit().await
let r = hub.spaces().setup(...).doit().await
```

The `resource()` and `activity(...)` calls create [builders][builder-pattern]. The second one dealing with `Activities`
supports various methods to configure the impending operation (not shown here). It is made such that all required arguments have to be
specified right away (i.e. `(...)`), whereas all optional ones can be [build up][builder-pattern] as desired.
The `doit()` method performs the actual communication with the server and returns the respective result.

# Usage

## Setting up your Project

To use this library, you would put the following lines into your `Cargo.toml` file:

```toml
[dependencies]
google-chat1 = "*"
serde = "1"
serde_json = "1"
```

## A complete example

```Rust
extern crate hyper;
extern crate hyper_rustls;
extern crate google_chat1 as chat1;
use chat1::{Result, Error};
use chat1::{HangoutsChat, FieldMask, hyper_rustls, hyper_util, yup_oauth2};

// Get an ApplicationSecret instance by some means. It contains the `client_id` and
// `client_secret`, among other things.
let secret: yup_oauth2::ApplicationSecret = Default::default();
// Instantiate the authenticator. It will choose a suitable authentication flow for you,
// unless you replace  `None` with the desired Flow.
// Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
// what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
// retrieve them from storage.
let connector = hyper_rustls::HttpsConnectorBuilder::new()
    .with_native_roots()
    .unwrap()
    .https_only()
    .enable_http2()
    .build();

let executor = hyper_util::rt::TokioExecutor::new();
let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
    secret,
    yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    yup_oauth2::client::CustomHyperClientBuilder::from(
        hyper_util::client::legacy::Client::builder(executor).build(connector),
    ),
).build().await.unwrap();

let client = hyper_util::client::legacy::Client::builder(
    hyper_util::rt::TokioExecutor::new()
)
.build(
    hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap()
        .https_or_http()
        .enable_http2()
        .build()
);
let mut hub = HangoutsChat::new(client, auth);
// You can configure optional parameters by calling the respective setters at will, and
// execute the final call using `doit()`.
// Values shown here are possibly random and not representative !
let result = hub.spaces().members_list("parent")
             .use_admin_access(true)
             .show_invited(false)
             .show_groups(true)
             .page_token("amet.")
             .page_size(-20)
             .filter("ipsum")
             .doit().await;

match result {
    Err(e) => match e {
        // The Error enum provides details about what exactly happened.
        // You can also just use its `Debug`, `Display` or `Error` traits
         Error::HttpError(_)
        |Error::Io(_)
        |Error::MissingAPIKey
        |Error::MissingToken(_)
        |Error::Cancelled
        |Error::UploadSizeLimitExceeded(_, _)
        |Error::Failure(_)
        |Error::BadRequest(_)
        |Error::FieldClash(_)
        |Error::JsonDecodeError(_, _) => println!("{}", e),
    },
    Ok(res) => println!("Success: {:?}", res),
}

```
## Handling Errors

All errors produced by the system are provided either as [Result](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Result) enumeration as return value of
the doit() methods, or handed as possibly intermediate results to either the
[Hub Delegate](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Delegate), or the [Authenticator Delegate](https://docs.rs/yup-oauth2/*/yup_oauth2/trait.AuthenticatorDelegate.html).

When delegates handle errors or intermediate values, they may have a chance to instruct the system to retry. This
makes the system potentially resilient to all kinds of errors.

## Uploads and Downloads
If a method supports downloads, the response body, which is part of the [Result](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Result), should be
read by you to obtain the media.
If such a method also supports a [Response Result](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::ResponseResult), it will return that by default.
You can see it as meta-data for the actual media. To trigger a media download, you will have to set up the builder by making
this call: `.param("alt", "media")`.

Methods supporting uploads can do so using up to 2 different protocols:
*simple* and *resumable*. The distinctiveness of each is represented by customized
`doit(...)` methods, which are then named `upload(...)` and `upload_resumable(...)` respectively.

## Customization and Callbacks

You may alter the way an `doit()` method is called by providing a [delegate](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Delegate) to the
[Method Builder](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::CallBuilder) before making the final `doit()` call.
Respective methods will be called to provide progress information, as well as determine whether the system should
retry on failure.

The [delegate trait](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Delegate) is default-implemented, allowing you to customize it with minimal effort.

## Optional Parts in Server-Requests

All structures provided by this library are made to be [encodable](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::RequestValue) and
[decodable](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::ResponseResult) via *json*. Optionals are used to indicate that partial requests are responses
are valid.
Most optionals are are considered [Parts](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::Part) which are identifiable by name, which will be sent to
the server to indicate either the set parts of the request or the desired parts in the response.

## Builder Arguments

Using [method builders](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::CallBuilder), you are able to prepare an action call by repeatedly calling it's methods.
These will always take a single argument, for which the following statements are true.

* [PODs][wiki-pod] are handed by copy
* strings are passed as `&str`
* [request values](https://docs.rs/google-chat1/7.0.0+20251214/google_chat1/common::RequestValue) are moved

Arguments will always be copied or cloned into the builder, to make them independent of their original life times.

[wiki-pod]: http://en.wikipedia.org/wiki/Plain_old_data_structure
[builder-pattern]: http://en.wikipedia.org/wiki/Builder_pattern
[google-go-api]: https://github.com/google/google-api-go-client

## Cargo Features

* `utoipa` - Add support for [utoipa](https://crates.io/crates/utoipa) and derive `utoipa::ToSchema` on all
the types. You'll have to import and register the required types in `#[openapi(schemas(...))]`, otherwise the
generated `openapi` spec would be invalid.


# License
The **chat1** library was generated by Sebastian Thiel, and is placed
under the *MIT* license.
You can read the full text at the repository's [license file][repo-license].

[repo-license]: https://github.com/Byron/google-apis-rsblob/main/LICENSE.md

