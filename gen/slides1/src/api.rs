#![allow(clippy::ptr_arg)]

use std::collections::{BTreeSet, HashMap};

use tokio::time::sleep;

// ##############
// UTILITIES ###
// ############

/// Identifies the an OAuth2 authorization scope.
/// A scope is needed when requesting an
/// [authorization token](https://developers.google.com/youtube/v3/guides/authentication).
#[derive(PartialEq, Eq, Ord, PartialOrd, Hash, Debug, Clone, Copy)]
pub enum Scope {
    /// See, edit, create, and delete all of your Google Drive files
    Drive,

    /// See, edit, create, and delete only the specific Google Drive files you use with this app
    DriveFile,

    /// See and download all your Google Drive files
    DriveReadonly,

    /// See, edit, create, and delete all your Google Slides presentations
    Presentation,

    /// See all your Google Slides presentations
    PresentationReadonly,

    /// See, edit, create, and delete all your Google Sheets spreadsheets
    Spreadsheet,

    /// See all your Google Sheets spreadsheets
    SpreadsheetReadonly,
}

impl AsRef<str> for Scope {
    fn as_ref(&self) -> &str {
        match *self {
            Scope::Drive => "https://www.googleapis.com/auth/drive",
            Scope::DriveFile => "https://www.googleapis.com/auth/drive.file",
            Scope::DriveReadonly => "https://www.googleapis.com/auth/drive.readonly",
            Scope::Presentation => "https://www.googleapis.com/auth/presentations",
            Scope::PresentationReadonly => "https://www.googleapis.com/auth/presentations.readonly",
            Scope::Spreadsheet => "https://www.googleapis.com/auth/spreadsheets",
            Scope::SpreadsheetReadonly => "https://www.googleapis.com/auth/spreadsheets.readonly",
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Scope {
    fn default() -> Scope {
        Scope::DriveReadonly
    }
}

// ########
// HUB ###
// ######

/// Central instance to access all Slides related resource activities
///
/// # Examples
///
/// Instantiate a new hub
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate google_slides1 as slides1;
/// use slides1::{Result, Error};
/// # async fn dox() {
/// use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// // Get an ApplicationSecret instance by some means. It contains the `client_id` and
/// // `client_secret`, among other things.
/// let secret: yup_oauth2::ApplicationSecret = Default::default();
/// // Instantiate the authenticator. It will choose a suitable authentication flow for you,
/// // unless you replace  `None` with the desired Flow.
/// // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
/// // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
/// // retrieve them from storage.
/// let connector = hyper_rustls::HttpsConnectorBuilder::new()
///     .with_native_roots()
///     .unwrap()
///     .https_only()
///     .enable_http2()
///     .build();
///
/// let executor = hyper_util::rt::TokioExecutor::new();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
///     secret,
///     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     yup_oauth2::client::CustomHyperClientBuilder::from(
///         hyper_util::client::legacy::Client::builder(executor).build(connector),
///     ),
/// ).build().await.unwrap();
///
/// let client = hyper_util::client::legacy::Client::builder(
///     hyper_util::rt::TokioExecutor::new()
/// )
/// .build(
///     hyper_rustls::HttpsConnectorBuilder::new()
///         .with_native_roots()
///         .unwrap()
///         .https_or_http()
///         .enable_http2()
///         .build()
/// );
/// let mut hub = Slides::new(client, auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.presentations().pages_get_thumbnail("presentationId", "pageObjectId")
///              .thumbnail_properties_thumbnail_size("amet.")
///              .thumbnail_properties_mime_type("duo")
///              .doit().await;
///
/// match result {
///     Err(e) => match e {
///         // The Error enum provides details about what exactly happened.
///         // You can also just use its `Debug`, `Display` or `Error` traits
///          Error::HttpError(_)
///         |Error::Io(_)
///         |Error::MissingAPIKey
///         |Error::MissingToken(_)
///         |Error::Cancelled
///         |Error::UploadSizeLimitExceeded(_, _)
///         |Error::Failure(_)
///         |Error::BadRequest(_)
///         |Error::FieldClash(_)
///         |Error::JsonDecodeError(_, _) => println!("{}", e),
///     },
///     Ok(res) => println!("Success: {:?}", res),
/// }
/// # }
/// ```
#[derive(Clone)]
pub struct Slides<C> {
    pub client: common::Client<C>,
    pub auth: Box<dyn common::GetToken>,
    _user_agent: String,
    _base_url: String,
    _root_url: String,
}

impl<C> common::Hub for Slides<C> {}

impl<'a, C> Slides<C> {
    pub fn new<A: 'static + common::GetToken>(client: common::Client<C>, auth: A) -> Slides<C> {
        Slides {
            client,
            auth: Box::new(auth),
            _user_agent: "google-api-rust-client/7.0.0".to_string(),
            _base_url: "https://slides.googleapis.com/".to_string(),
            _root_url: "https://slides.googleapis.com/".to_string(),
        }
    }

    pub fn presentations(&'a self) -> PresentationMethods<'a, C> {
        PresentationMethods { hub: self }
    }

    /// Set the user-agent header field to use in all requests to the server.
    /// It defaults to `google-api-rust-client/7.0.0`.
    ///
    /// Returns the previously set user-agent.
    pub fn user_agent(&mut self, agent_name: String) -> String {
        std::mem::replace(&mut self._user_agent, agent_name)
    }

    /// Set the base url to use in all requests to the server.
    /// It defaults to `https://slides.googleapis.com/`.
    ///
    /// Returns the previously set base url.
    pub fn base_url(&mut self, new_base_url: String) -> String {
        std::mem::replace(&mut self._base_url, new_base_url)
    }

    /// Set the root url to use in all requests to the server.
    /// It defaults to `https://slides.googleapis.com/`.
    ///
    /// Returns the previously set root url.
    pub fn root_url(&mut self, new_root_url: String) -> String {
        std::mem::replace(&mut self._root_url, new_root_url)
    }
}

// ############
// SCHEMAS ###
// ##########
/// AffineTransform uses a 3x3 matrix with an implied last row of [ 0 0 1 ] to transform source coordinates (x,y) into destination coordinates (x', y') according to: x' x = shear_y scale_y translate_y 1 [ 1 ] After transformation, x' = scale_x * x + shear_x * y + translate_x; y' = scale_y * y + shear_y * x + translate_y; This message is therefore composed of these six matrix elements.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AffineTransform {
    /// The X coordinate scaling element.
    #[serde(rename = "scaleX")]
    pub scale_x: Option<f64>,
    /// The Y coordinate scaling element.
    #[serde(rename = "scaleY")]
    pub scale_y: Option<f64>,
    /// The X coordinate shearing element.
    #[serde(rename = "shearX")]
    pub shear_x: Option<f64>,
    /// The Y coordinate shearing element.
    #[serde(rename = "shearY")]
    pub shear_y: Option<f64>,
    /// The X coordinate translation element.
    #[serde(rename = "translateX")]
    pub translate_x: Option<f64>,
    /// The Y coordinate translation element.
    #[serde(rename = "translateY")]
    pub translate_y: Option<f64>,
    /// The units for translate elements.
    pub unit: Option<String>,
}

impl common::Part for AffineTransform {}

/// A TextElement kind that represents auto text.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AutoText {
    /// The rendered content of this auto text, if available.
    pub content: Option<String>,
    /// The styling applied to this auto text.
    pub style: Option<TextStyle>,
    /// The type of this auto text.
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl common::Part for AutoText {}

/// The autofit properties of a Shape. This property is only set for shapes that allow text.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Autofit {
    /// The autofit type of the shape. If the autofit type is AUTOFIT_TYPE_UNSPECIFIED, the autofit type is inherited from a parent placeholder if it exists. The field is automatically set to NONE if a request is made that might affect text fitting within its bounding text box. In this case, the font_scale is applied to the font_size and the line_spacing_reduction is applied to the line_spacing. Both properties are also reset to default values.
    #[serde(rename = "autofitType")]
    pub autofit_type: Option<String>,
    /// The font scale applied to the shape. For shapes with autofit_type NONE or SHAPE_AUTOFIT, this value is the default value of 1. For TEXT_AUTOFIT, this value multiplied by the font_size gives the font size that's rendered in the editor. This property is read-only.
    #[serde(rename = "fontScale")]
    pub font_scale: Option<f32>,
    /// The line spacing reduction applied to the shape. For shapes with autofit_type NONE or SHAPE_AUTOFIT, this value is the default value of 0. For TEXT_AUTOFIT, this value subtracted from the line_spacing gives the line spacing that's rendered in the editor. This property is read-only.
    #[serde(rename = "lineSpacingReduction")]
    pub line_spacing_reduction: Option<f32>,
}

impl common::Part for Autofit {}

/// Request message for PresentationsService.BatchUpdatePresentation.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [batch update presentations](PresentationBatchUpdateCall) (request)
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BatchUpdatePresentationRequest {
    /// A list of updates to apply to the presentation.
    pub requests: Option<Vec<Request>>,
    /// Provides control over how write requests are executed.
    #[serde(rename = "writeControl")]
    pub write_control: Option<WriteControl>,
}

impl common::RequestValue for BatchUpdatePresentationRequest {}

/// Response message from a batch update.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [batch update presentations](PresentationBatchUpdateCall) (response)
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BatchUpdatePresentationResponse {
    /// The presentation the updates were applied to.
    #[serde(rename = "presentationId")]
    pub presentation_id: Option<String>,
    /// The reply of the updates. This maps 1:1 with the updates, although replies to some requests may be empty.
    pub replies: Option<Vec<Response>>,
    /// The updated write control after applying the request.
    #[serde(rename = "writeControl")]
    pub write_control: Option<WriteControl>,
}

impl common::ResponseResult for BatchUpdatePresentationResponse {}

/// Describes the bullet of a paragraph.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Bullet {
    /// The paragraph specific text style applied to this bullet.
    #[serde(rename = "bulletStyle")]
    pub bullet_style: Option<TextStyle>,
    /// The rendered bullet glyph for this paragraph.
    pub glyph: Option<String>,
    /// The ID of the list this paragraph belongs to.
    #[serde(rename = "listId")]
    pub list_id: Option<String>,
    /// The nesting level of this paragraph in the list.
    #[serde(rename = "nestingLevel")]
    pub nesting_level: Option<i32>,
}

impl common::Part for Bullet {}

/// The palette of predefined colors for a page.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ColorScheme {
    /// The ThemeColorType and corresponding concrete color pairs.
    pub colors: Option<Vec<ThemeColorPair>>,
}

impl common::Part for ColorScheme {}

/// A color and position in a gradient band.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ColorStop {
    /// The alpha value of this color in the gradient band. Defaults to 1.0, fully opaque.
    pub alpha: Option<f32>,
    /// The color of the gradient stop.
    pub color: Option<OpaqueColor>,
    /// The relative position of the color stop in the gradient band measured in percentage. The value should be in the interval [0.0, 1.0].
    pub position: Option<f32>,
}

impl common::Part for ColorStop {}

/// Creates an image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateImageRequest {
    /// The element properties for the image. When the aspect ratio of the provided size does not match the image aspect ratio, the image is scaled and centered with respect to the size in order to maintain the aspect ratio. The provided transform is applied after this operation. The PageElementProperties.size property is optional. If you don't specify the size, the default size of the image is used. The PageElementProperties.transform property is optional. If you don't specify a transform, the image will be placed at the top-left corner of the page.
    #[serde(rename = "elementProperties")]
    pub element_properties: Option<PageElementProperties>,
    /// A user-supplied object ID. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The image URL. The image is fetched once at insertion time and a copy is stored for display inside the presentation. Images must be less than 50 MB in size, can't exceed 25 megapixels, and must be in one of PNG, JPEG, or GIF formats. The provided URL must be publicly accessible and up to 2 KB in length. The URL is saved with the image, and exposed through the Image.source_url field.
    pub url: Option<String>,
}

impl common::Part for CreateImageRequest {}

/// The result of creating an image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateImageResponse {
    /// The object ID of the created image.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateImageResponse {}

/// Creates a line.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateLineRequest {
    /// The category of the line to be created. The exact line type created is determined based on the category and how it's routed to connect to other page elements. If you specify both a `category` and a `line_category`, the `category` takes precedence. If you do not specify a value for `category`, but specify a value for `line_category`, then the specified `line_category` value is used. If you do not specify either, then STRAIGHT is used.
    pub category: Option<String>,
    /// The element properties for the line.
    #[serde(rename = "elementProperties")]
    pub element_properties: Option<PageElementProperties>,
    /// The category of the line to be created. *Deprecated*: use `category` instead. The exact line type created is determined based on the category and how it's routed to connect to other page elements. If you specify both a `category` and a `line_category`, the `category` takes precedence.
    #[serde(rename = "lineCategory")]
    pub line_category: Option<String>,
    /// A user-supplied object ID. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateLineRequest {}

/// The result of creating a line.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateLineResponse {
    /// The object ID of the created line.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateLineResponse {}

/// Creates bullets for all of the paragraphs that overlap with the given text index range. The nesting level of each paragraph will be determined by counting leading tabs in front of each paragraph. To avoid excess space between the bullet and the corresponding paragraph, these leading tabs are removed by this request. This may change the indices of parts of the text. If the paragraph immediately before paragraphs being updated is in a list with a matching preset, the paragraphs being updated are added to that preceding list.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateParagraphBulletsRequest {
    /// The kinds of bullet glyphs to be used. Defaults to the `BULLET_DISC_CIRCLE_SQUARE` preset.
    #[serde(rename = "bulletPreset")]
    pub bullet_preset: Option<String>,
    /// The optional table cell location if the text to be modified is in a table cell. If present, the object_id must refer to a table.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The object ID of the shape or table containing the text to add bullets to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The range of text to apply the bullet presets to, based on TextElement indexes.
    #[serde(rename = "textRange")]
    pub text_range: Option<Range>,
}

impl common::Part for CreateParagraphBulletsRequest {}

/// Creates a new shape.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateShapeRequest {
    /// The element properties for the shape.
    #[serde(rename = "elementProperties")]
    pub element_properties: Option<PageElementProperties>,
    /// A user-supplied object ID. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If empty, a unique identifier will be generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The shape type.
    #[serde(rename = "shapeType")]
    pub shape_type: Option<String>,
}

impl common::Part for CreateShapeRequest {}

/// The result of creating a shape.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateShapeResponse {
    /// The object ID of the created shape.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateShapeResponse {}

/// Creates an embedded Google Sheets chart. NOTE: Chart creation requires at least one of the spreadsheets.readonly, spreadsheets, drive.readonly, drive.file, or drive OAuth scopes.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSheetsChartRequest {
    /// The ID of the specific chart in the Google Sheets spreadsheet.
    #[serde(rename = "chartId")]
    pub chart_id: Option<i32>,
    /// The element properties for the chart. When the aspect ratio of the provided size does not match the chart aspect ratio, the chart is scaled and centered with respect to the size in order to maintain aspect ratio. The provided transform is applied after this operation.
    #[serde(rename = "elementProperties")]
    pub element_properties: Option<PageElementProperties>,
    /// The mode with which the chart is linked to the source spreadsheet. When not specified, the chart will be an image that is not linked.
    #[serde(rename = "linkingMode")]
    pub linking_mode: Option<String>,
    /// A user-supplied object ID. If specified, the ID must be unique among all pages and page elements in the presentation. The ID should start with a word character [a-zA-Z0-9_] and then followed by any number of the following characters [a-zA-Z0-9_-:]. The length of the ID should not be less than 5 or greater than 50. If empty, a unique identifier will be generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The ID of the Google Sheets spreadsheet that contains the chart. You might need to add a resource key to the HTTP header for a subset of old files. For more information, see [Access link-shared files using resource keys](https://developers.google.com/drive/api/v3/resource-keys).
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: Option<String>,
}

impl common::Part for CreateSheetsChartRequest {}

/// The result of creating an embedded Google Sheets chart.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSheetsChartResponse {
    /// The object ID of the created chart.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateSheetsChartResponse {}

/// Creates a slide.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSlideRequest {
    /// The optional zero-based index indicating where to insert the slides. If you don't specify an index, the slide is created at the end.
    #[serde(rename = "insertionIndex")]
    pub insertion_index: Option<i32>,
    /// A user-supplied object ID. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The ID length must be between 5 and 50 characters, inclusive. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// An optional list of object ID mappings from the placeholder(s) on the layout to the placeholders that are created on the slide from the specified layout. Can only be used when `slide_layout_reference` is specified.
    #[serde(rename = "placeholderIdMappings")]
    pub placeholder_id_mappings: Option<Vec<LayoutPlaceholderIdMapping>>,
    /// Layout reference of the slide to be inserted, based on the *current master*, which is one of the following: - The master of the previous slide index. - The master of the first slide, if the insertion_index is zero. - The first master in the presentation, if there are no slides. If the LayoutReference is not found in the current master, a 400 bad request error is returned. If you don't specify a layout reference, the slide uses the predefined `BLANK` layout.
    #[serde(rename = "slideLayoutReference")]
    pub slide_layout_reference: Option<LayoutReference>,
}

impl common::Part for CreateSlideRequest {}

/// The result of creating a slide.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSlideResponse {
    /// The object ID of the created slide.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateSlideResponse {}

/// Creates a new table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateTableRequest {
    /// Number of columns in the table.
    pub columns: Option<i32>,
    /// The element properties for the table. The table will be created at the provided size, subject to a minimum size. If no size is provided, the table will be automatically sized. Table transforms must have a scale of 1 and no shear components. If no transform is provided, the table will be centered on the page.
    #[serde(rename = "elementProperties")]
    pub element_properties: Option<PageElementProperties>,
    /// A user-supplied object ID. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// Number of rows in the table.
    pub rows: Option<i32>,
}

impl common::Part for CreateTableRequest {}

/// The result of creating a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateTableResponse {
    /// The object ID of the created table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateTableResponse {}

/// Creates a video. NOTE: Creating a video from Google Drive requires that the requesting app have at least one of the drive, drive.readonly, or drive.file OAuth scopes.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateVideoRequest {
    /// The element properties for the video. The PageElementProperties.size property is optional. If you don't specify a size, a default size is chosen by the server. The PageElementProperties.transform property is optional. The transform must not have shear components. If you don't specify a transform, the video will be placed at the top left corner of the page.
    #[serde(rename = "elementProperties")]
    pub element_properties: Option<PageElementProperties>,
    /// The video source's unique identifier for this video. e.g. For YouTube video https://www.youtube.com/watch?v=7U3axjORYZ0, the ID is 7U3axjORYZ0. For a Google Drive video https://drive.google.com/file/d/1xCgQLFTJi5_Xl8DgW_lcUYq5e-q6Hi5Q the ID is 1xCgQLFTJi5_Xl8DgW_lcUYq5e-q6Hi5Q. To access a Google Drive video file, you might need to add a resource key to the HTTP header for a subset of old files. For more information, see [Access link-shared files using resource keys](https://developers.google.com/drive/api/v3/resource-keys).
    pub id: Option<String>,
    /// A user-supplied object ID. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The video source.
    pub source: Option<String>,
}

impl common::Part for CreateVideoRequest {}

/// The result of creating a video.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateVideoResponse {
    /// The object ID of the created video.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for CreateVideoResponse {}

/// The crop properties of an object enclosed in a container. For example, an Image. The crop properties is represented by the offsets of four edges which define a crop rectangle. The offsets are measured in percentage from the corresponding edges of the object's original bounding rectangle towards inside, relative to the object's original dimensions. - If the offset is in the interval (0, 1), the corresponding edge of crop rectangle is positioned inside of the object's original bounding rectangle. - If the offset is negative or greater than 1, the corresponding edge of crop rectangle is positioned outside of the object's original bounding rectangle. - If the left edge of the crop rectangle is on the right side of its right edge, the object will be flipped horizontally. - If the top edge of the crop rectangle is below its bottom edge, the object will be flipped vertically. - If all offsets and rotation angle is 0, the object is not cropped. After cropping, the content in the crop rectangle will be stretched to fit its container.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CropProperties {
    /// The rotation angle of the crop window around its center, in radians. Rotation angle is applied after the offset.
    pub angle: Option<f32>,
    /// The offset specifies the bottom edge of the crop rectangle that is located above the original bounding rectangle bottom edge, relative to the object's original height.
    #[serde(rename = "bottomOffset")]
    pub bottom_offset: Option<f32>,
    /// The offset specifies the left edge of the crop rectangle that is located to the right of the original bounding rectangle left edge, relative to the object's original width.
    #[serde(rename = "leftOffset")]
    pub left_offset: Option<f32>,
    /// The offset specifies the right edge of the crop rectangle that is located to the left of the original bounding rectangle right edge, relative to the object's original width.
    #[serde(rename = "rightOffset")]
    pub right_offset: Option<f32>,
    /// The offset specifies the top edge of the crop rectangle that is located below the original bounding rectangle top edge, relative to the object's original height.
    #[serde(rename = "topOffset")]
    pub top_offset: Option<f32>,
}

impl common::Part for CropProperties {}

/// Deletes an object, either pages or page elements, from the presentation.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteObjectRequest {
    /// The object ID of the page or page element to delete. If after a delete operation a group contains only 1 or no page elements, the group is also deleted. If a placeholder is deleted on a layout, any empty inheriting placeholders are also deleted.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for DeleteObjectRequest {}

/// Deletes bullets from all of the paragraphs that overlap with the given text index range. The nesting level of each paragraph will be visually preserved by adding indent to the start of the corresponding paragraph.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteParagraphBulletsRequest {
    /// The optional table cell location if the text to be modified is in a table cell. If present, the object_id must refer to a table.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The object ID of the shape or table containing the text to delete bullets from.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The range of text to delete bullets from, based on TextElement indexes.
    #[serde(rename = "textRange")]
    pub text_range: Option<Range>,
}

impl common::Part for DeleteParagraphBulletsRequest {}

/// Deletes a column from a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteTableColumnRequest {
    /// The reference table cell location from which a column will be deleted. The column this cell spans will be deleted. If this is a merged cell, multiple columns will be deleted. If no columns remain in the table after this deletion, the whole table is deleted.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The table to delete columns from.
    #[serde(rename = "tableObjectId")]
    pub table_object_id: Option<String>,
}

impl common::Part for DeleteTableColumnRequest {}

/// Deletes a row from a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteTableRowRequest {
    /// The reference table cell location from which a row will be deleted. The row this cell spans will be deleted. If this is a merged cell, multiple rows will be deleted. If no rows remain in the table after this deletion, the whole table is deleted.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The table to delete rows from.
    #[serde(rename = "tableObjectId")]
    pub table_object_id: Option<String>,
}

impl common::Part for DeleteTableRowRequest {}

/// Deletes text from a shape or a table cell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteTextRequest {
    /// The optional table cell location if the text is to be deleted from a table cell. If present, the object_id must refer to a table.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The object ID of the shape or table from which the text will be deleted.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The range of text to delete, based on TextElement indexes. There is always an implicit newline character at the end of a shape's or table cell's text that cannot be deleted. `Range.Type.ALL` will use the correct bounds, but care must be taken when specifying explicit bounds for range types `FROM_START_INDEX` and `FIXED_RANGE`. For example, if the text is "ABC", followed by an implicit newline, then the maximum value is 2 for `text_range.start_index` and 3 for `text_range.end_index`. Deleting text that crosses a paragraph boundary may result in changes to paragraph styles and lists as the two paragraphs are merged. Ranges that include only one code unit of a surrogate pair are expanded to include both code units.
    #[serde(rename = "textRange")]
    pub text_range: Option<Range>,
}

impl common::Part for DeleteTextRequest {}

/// A magnitude in a single direction in the specified units.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Dimension {
    /// The magnitude.
    pub magnitude: Option<f64>,
    /// The units for magnitude.
    pub unit: Option<String>,
}

impl common::Part for Dimension {}

/// Duplicates a slide or page element. When duplicating a slide, the duplicate slide will be created immediately following the specified slide. When duplicating a page element, the duplicate will be placed on the same page at the same position as the original.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DuplicateObjectRequest {
    /// The ID of the object to duplicate.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The object being duplicated may contain other objects, for example when duplicating a slide or a group page element. This map defines how the IDs of duplicated objects are generated: the keys are the IDs of the original objects and its values are the IDs that will be assigned to the corresponding duplicate object. The ID of the source object's duplicate may be specified in this map as well, using the same value of the `object_id` field as a key and the newly desired ID as the value. All keys must correspond to existing IDs in the presentation. All values must be unique in the presentation and must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the new ID must not be less than 5 or greater than 50. If any IDs of source objects are omitted from the map, a new random ID will be assigned. If the map is empty or unset, all duplicate objects will receive a new random ID.
    #[serde(rename = "objectIds")]
    pub object_ids: Option<HashMap<String, String>>,
}

impl common::Part for DuplicateObjectRequest {}

/// The response of duplicating an object.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DuplicateObjectResponse {
    /// The ID of the new duplicate object.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for DuplicateObjectResponse {}

/// A PageElement kind representing a joined collection of PageElements.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Group {
    /// The collection of elements in the group. The minimum size of a group is 2.
    pub children: Option<Vec<Option<Box<PageElement>>>>,
}

impl common::Part for Group {}

/// Groups objects to create an object group. For example, groups PageElements to create a Group on the same page as all the children.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GroupObjectsRequest {
    /// The object IDs of the objects to group. Only page elements can be grouped. There should be at least two page elements on the same page that are not already in another group. Some page elements, such as videos, tables and placeholders cannot be grouped.
    #[serde(rename = "childrenObjectIds")]
    pub children_object_ids: Option<Vec<String>>,
    /// A user-supplied object ID for the group to be created. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "groupObjectId")]
    pub group_object_id: Option<String>,
}

impl common::Part for GroupObjectsRequest {}

/// The result of grouping objects.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GroupObjectsResponse {
    /// The object ID of the created group.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for GroupObjectsResponse {}

/// A PageElement kind representing an image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Image {
    /// An URL to an image with a default lifetime of 30 minutes. This URL is tagged with the account of the requester. Anyone with the URL effectively accesses the image as the original requester. Access to the image may be lost if the presentation's sharing settings change.
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
    /// The properties of the image.
    #[serde(rename = "imageProperties")]
    pub image_properties: Option<ImageProperties>,
    /// Placeholders are page elements that inherit from corresponding placeholders on layouts and masters. If set, the image is a placeholder image and any inherited properties can be resolved by looking at the parent placeholder identified by the Placeholder.parent_object_id field.
    pub placeholder: Option<Placeholder>,
    /// The source URL is the URL used to insert the image. The source URL can be empty.
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
}

impl common::Part for Image {}

/// The properties of the Image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageProperties {
    /// The brightness effect of the image. The value should be in the interval [-1.0, 1.0], where 0 means no effect. This property is read-only.
    pub brightness: Option<f32>,
    /// The contrast effect of the image. The value should be in the interval [-1.0, 1.0], where 0 means no effect. This property is read-only.
    pub contrast: Option<f32>,
    /// The crop properties of the image. If not set, the image is not cropped. This property is read-only.
    #[serde(rename = "cropProperties")]
    pub crop_properties: Option<CropProperties>,
    /// The hyperlink destination of the image. If unset, there is no link.
    pub link: Option<Link>,
    /// The outline of the image. If not set, the image has no outline.
    pub outline: Option<Outline>,
    /// The recolor effect of the image. If not set, the image is not recolored. This property is read-only.
    pub recolor: Option<Recolor>,
    /// The shadow of the image. If not set, the image has no shadow. This property is read-only.
    pub shadow: Option<Shadow>,
    /// The transparency effect of the image. The value should be in the interval [0.0, 1.0], where 0 means no effect and 1 means completely transparent. This property is read-only.
    pub transparency: Option<f32>,
}

impl common::Part for ImageProperties {}

/// Inserts columns into a table. Other columns in the table will be resized to fit the new column.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InsertTableColumnsRequest {
    /// The reference table cell location from which columns will be inserted. A new column will be inserted to the left (or right) of the column where the reference cell is. If the reference cell is a merged cell, a new column will be inserted to the left (or right) of the merged cell.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// Whether to insert new columns to the right of the reference cell location. - `True`: insert to the right. - `False`: insert to the left.
    #[serde(rename = "insertRight")]
    pub insert_right: Option<bool>,
    /// The number of columns to be inserted. Maximum 20 per request.
    pub number: Option<i32>,
    /// The table to insert columns into.
    #[serde(rename = "tableObjectId")]
    pub table_object_id: Option<String>,
}

impl common::Part for InsertTableColumnsRequest {}

/// Inserts rows into a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InsertTableRowsRequest {
    /// The reference table cell location from which rows will be inserted. A new row will be inserted above (or below) the row where the reference cell is. If the reference cell is a merged cell, a new row will be inserted above (or below) the merged cell.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// Whether to insert new rows below the reference cell location. - `True`: insert below the cell. - `False`: insert above the cell.
    #[serde(rename = "insertBelow")]
    pub insert_below: Option<bool>,
    /// The number of rows to be inserted. Maximum 20 per request.
    pub number: Option<i32>,
    /// The table to insert rows into.
    #[serde(rename = "tableObjectId")]
    pub table_object_id: Option<String>,
}

impl common::Part for InsertTableRowsRequest {}

/// Inserts text into a shape or a table cell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InsertTextRequest {
    /// The optional table cell location if the text is to be inserted into a table cell. If present, the object_id must refer to a table.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The index where the text will be inserted, in Unicode code units, based on TextElement indexes. The index is zero-based and is computed from the start of the string. The index may be adjusted to prevent insertions inside Unicode grapheme clusters. In these cases, the text will be inserted immediately after the grapheme cluster.
    #[serde(rename = "insertionIndex")]
    pub insertion_index: Option<i32>,
    /// The object ID of the shape or table where the text will be inserted.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The text to be inserted. Inserting a newline character will implicitly create a new ParagraphMarker at that index. The paragraph style of the new paragraph will be copied from the paragraph at the current insertion index, including lists and bullets. Text styles for inserted text will be determined automatically, generally preserving the styling of neighboring text. In most cases, the text will be added to the TextRun that exists at the insertion index. Some control characters (U+0000-U+0008, U+000C-U+001F) and characters from the Unicode Basic Multilingual Plane Private Use Area (U+E000-U+F8FF) will be stripped out of the inserted text.
    pub text: Option<String>,
}

impl common::Part for InsertTextRequest {}

/// The user-specified ID mapping for a placeholder that will be created on a slide from a specified layout.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayoutPlaceholderIdMapping {
    /// The placeholder on a layout that will be applied to a slide. Only type and index are needed. For example, a predefined `TITLE_AND_BODY` layout may usually have a TITLE placeholder with index 0 and a BODY placeholder with index 0.
    #[serde(rename = "layoutPlaceholder")]
    pub layout_placeholder: Option<Placeholder>,
    /// The object ID of the placeholder on a layout that will be applied to a slide.
    #[serde(rename = "layoutPlaceholderObjectId")]
    pub layout_placeholder_object_id: Option<String>,
    /// A user-supplied object ID for the placeholder identified above that to be created onto a slide. If you specify an ID, it must be unique among all pages and page elements in the presentation. The ID must start with an alphanumeric character or an underscore (matches regex `[a-zA-Z0-9_]`); remaining characters may include those as well as a hyphen or colon (matches regex `[a-zA-Z0-9_-:]`). The length of the ID must not be less than 5 or greater than 50. If you don't specify an ID, a unique one is generated.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for LayoutPlaceholderIdMapping {}

/// The properties of Page are only relevant for pages with page_type LAYOUT.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayoutProperties {
    /// The human-readable name of the layout.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    /// The object ID of the master that this layout is based on.
    #[serde(rename = "masterObjectId")]
    pub master_object_id: Option<String>,
    /// The name of the layout.
    pub name: Option<String>,
}

impl common::Part for LayoutProperties {}

/// Slide layout reference. This may reference either: - A predefined layout - One of the layouts in the presentation.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayoutReference {
    /// Layout ID: the object ID of one of the layouts in the presentation.
    #[serde(rename = "layoutId")]
    pub layout_id: Option<String>,
    /// Predefined layout.
    #[serde(rename = "predefinedLayout")]
    pub predefined_layout: Option<String>,
}

impl common::Part for LayoutReference {}

/// A PageElement kind representing a non-connector line, straight connector, curved connector, or bent connector.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Line {
    /// The category of the line. It matches the `category` specified in CreateLineRequest, and can be updated with UpdateLineCategoryRequest.
    #[serde(rename = "lineCategory")]
    pub line_category: Option<String>,
    /// The properties of the line.
    #[serde(rename = "lineProperties")]
    pub line_properties: Option<LineProperties>,
    /// The type of the line.
    #[serde(rename = "lineType")]
    pub line_type: Option<String>,
}

impl common::Part for Line {}

/// The properties for one end of a Line connection.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LineConnection {
    /// The object ID of the connected page element. Some page elements, such as groups, tables, and lines do not have connection sites and therefore cannot be connected to a connector line.
    #[serde(rename = "connectedObjectId")]
    pub connected_object_id: Option<String>,
    /// The index of the connection site on the connected page element. In most cases, it corresponds to the predefined connection site index from the ECMA-376 standard. More information on those connection sites can be found in both the description of the "cxn" attribute in section 20.1.9.9 and "Annex H. Example Predefined DrawingML Shape and Text Geometries" of "Office Open XML File Formats - Fundamentals and Markup Language Reference", part 1 of [ECMA-376 5th edition](https://ecma-international.org/publications-and-standards/standards/ecma-376/). The position of each connection site can also be viewed from Slides editor.
    #[serde(rename = "connectionSiteIndex")]
    pub connection_site_index: Option<i32>,
}

impl common::Part for LineConnection {}

/// The fill of the line.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LineFill {
    /// Solid color fill.
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<SolidFill>,
}

impl common::Part for LineFill {}

/// The properties of the Line. When unset, these fields default to values that match the appearance of new lines created in the Slides editor.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LineProperties {
    /// The dash style of the line.
    #[serde(rename = "dashStyle")]
    pub dash_style: Option<String>,
    /// The style of the arrow at the end of the line.
    #[serde(rename = "endArrow")]
    pub end_arrow: Option<String>,
    /// The connection at the end of the line. If unset, there is no connection. Only lines with a Type indicating it is a "connector" can have an `end_connection`.
    #[serde(rename = "endConnection")]
    pub end_connection: Option<LineConnection>,
    /// The fill of the line. The default line fill matches the defaults for new lines created in the Slides editor.
    #[serde(rename = "lineFill")]
    pub line_fill: Option<LineFill>,
    /// The hyperlink destination of the line. If unset, there is no link.
    pub link: Option<Link>,
    /// The style of the arrow at the beginning of the line.
    #[serde(rename = "startArrow")]
    pub start_arrow: Option<String>,
    /// The connection at the beginning of the line. If unset, there is no connection. Only lines with a Type indicating it is a "connector" can have a `start_connection`.
    #[serde(rename = "startConnection")]
    pub start_connection: Option<LineConnection>,
    /// The thickness of the line.
    pub weight: Option<Dimension>,
}

impl common::Part for LineProperties {}

/// A hypertext link.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Link {
    /// If set, indicates this is a link to the specific page in this presentation with this ID. A page with this ID may not exist.
    #[serde(rename = "pageObjectId")]
    pub page_object_id: Option<String>,
    /// If set, indicates this is a link to a slide in this presentation, addressed by its position.
    #[serde(rename = "relativeLink")]
    pub relative_link: Option<String>,
    /// If set, indicates this is a link to the slide at this zero-based index in the presentation. There may not be a slide at this index.
    #[serde(rename = "slideIndex")]
    pub slide_index: Option<i32>,
    /// If set, indicates this is a link to the external web page at this URL.
    pub url: Option<String>,
}

impl common::Part for Link {}

/// A List describes the look and feel of bullets belonging to paragraphs associated with a list. A paragraph that is part of a list has an implicit reference to that list's ID.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct List {
    /// The ID of the list.
    #[serde(rename = "listId")]
    pub list_id: Option<String>,
    /// A map of nesting levels to the properties of bullets at the associated level. A list has at most nine levels of nesting, so the possible values for the keys of this map are 0 through 8, inclusive.
    #[serde(rename = "nestingLevel")]
    pub nesting_level: Option<HashMap<String, NestingLevel>>,
}

impl common::Part for List {}

/// The properties of Page that are only relevant for pages with page_type MASTER.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MasterProperties {
    /// The human-readable name of the master.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

impl common::Part for MasterProperties {}

/// Merges cells in a Table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MergeTableCellsRequest {
    /// The object ID of the table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The table range specifying which cells of the table to merge. Any text in the cells being merged will be concatenated and stored in the upper-left ("head") cell of the range. If the range is non-rectangular (which can occur in some cases where the range covers cells that are already merged), a 400 bad request error is returned.
    #[serde(rename = "tableRange")]
    pub table_range: Option<TableRange>,
}

impl common::Part for MergeTableCellsRequest {}

/// Contains properties describing the look and feel of a list bullet at a given level of nesting.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NestingLevel {
    /// The style of a bullet at this level of nesting.
    #[serde(rename = "bulletStyle")]
    pub bullet_style: Option<TextStyle>,
}

impl common::Part for NestingLevel {}

/// The properties of Page that are only relevant for pages with page_type NOTES.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NotesProperties {
    /// The object ID of the shape on this notes page that contains the speaker notes for the corresponding slide. The actual shape may not always exist on the notes page. Inserting text using this object ID will automatically create the shape. In this case, the actual shape may have different object ID. The `GetPresentation` or `GetPage` action will always return the latest object ID.
    #[serde(rename = "speakerNotesObjectId")]
    pub speaker_notes_object_id: Option<String>,
}

impl common::Part for NotesProperties {}

/// A themeable solid color value.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpaqueColor {
    /// An opaque RGB color.
    #[serde(rename = "rgbColor")]
    pub rgb_color: Option<RgbColor>,
    /// An opaque theme color.
    #[serde(rename = "themeColor")]
    pub theme_color: Option<String>,
}

impl common::Part for OpaqueColor {}

/// A color that can either be fully opaque or fully transparent.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OptionalColor {
    /// If set, this will be used as an opaque color. If unset, this represents a transparent color.
    #[serde(rename = "opaqueColor")]
    pub opaque_color: Option<OpaqueColor>,
}

impl common::Part for OptionalColor {}

/// The outline of a PageElement. If these fields are unset, they may be inherited from a parent placeholder if it exists. If there is no parent, the fields will default to the value used for new page elements created in the Slides editor, which may depend on the page element kind.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Outline {
    /// The dash style of the outline.
    #[serde(rename = "dashStyle")]
    pub dash_style: Option<String>,
    /// The fill of the outline.
    #[serde(rename = "outlineFill")]
    pub outline_fill: Option<OutlineFill>,
    /// The outline property state. Updating the outline on a page element will implicitly update this field to `RENDERED`, unless another value is specified in the same request. To have no outline on a page element, set this field to `NOT_RENDERED`. In this case, any other outline fields set in the same request will be ignored.
    #[serde(rename = "propertyState")]
    pub property_state: Option<String>,
    /// The thickness of the outline.
    pub weight: Option<Dimension>,
}

impl common::Part for Outline {}

/// The fill of the outline.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OutlineFill {
    /// Solid color fill.
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<SolidFill>,
}

impl common::Part for OutlineFill {}

/// A page in a presentation.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [pages get presentations](PresentationPageGetCall) (response)
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Page {
    /// Layout specific properties. Only set if page_type = LAYOUT.
    #[serde(rename = "layoutProperties")]
    pub layout_properties: Option<LayoutProperties>,
    /// Master specific properties. Only set if page_type = MASTER.
    #[serde(rename = "masterProperties")]
    pub master_properties: Option<MasterProperties>,
    /// Notes specific properties. Only set if page_type = NOTES.
    #[serde(rename = "notesProperties")]
    pub notes_properties: Option<NotesProperties>,
    /// The object ID for this page. Object IDs used by Page and PageElement share the same namespace.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The page elements rendered on the page.
    #[serde(rename = "pageElements")]
    pub page_elements: Option<Vec<PageElement>>,
    /// The properties of the page.
    #[serde(rename = "pageProperties")]
    pub page_properties: Option<PageProperties>,
    /// The type of the page.
    #[serde(rename = "pageType")]
    pub page_type: Option<String>,
    /// Output only. The revision ID of the presentation. Can be used in update requests to assert the presentation revision hasn't changed since the last read operation. Only populated if the user has edit access to the presentation. The revision ID is not a sequential number but an opaque string. The format of the revision ID might change over time. A returned revision ID is only guaranteed to be valid for 24 hours after it has been returned and cannot be shared across users. If the revision ID is unchanged between calls, then the presentation has not changed. Conversely, a changed ID (for the same presentation and user) usually means the presentation has been updated. However, a changed ID can also be due to internal factors such as ID format changes.
    #[serde(rename = "revisionId")]
    pub revision_id: Option<String>,
    /// Slide specific properties. Only set if page_type = SLIDE.
    #[serde(rename = "slideProperties")]
    pub slide_properties: Option<Box<SlideProperties>>,
}

impl common::ResponseResult for Page {}

/// The page background fill.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PageBackgroundFill {
    /// The background fill property state. Updating the fill on a page will implicitly update this field to `RENDERED`, unless another value is specified in the same request. To have no fill on a page, set this field to `NOT_RENDERED`. In this case, any other fill fields set in the same request will be ignored.
    #[serde(rename = "propertyState")]
    pub property_state: Option<String>,
    /// Solid color fill.
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<SolidFill>,
    /// Stretched picture fill.
    #[serde(rename = "stretchedPictureFill")]
    pub stretched_picture_fill: Option<StretchedPictureFill>,
}

impl common::Part for PageBackgroundFill {}

/// A visual element rendered on a page.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PageElement {
    /// The description of the page element. Combined with title to display alt text. The field is not supported for Group elements.
    pub description: Option<String>,
    /// A collection of page elements joined as a single unit.
    #[serde(rename = "elementGroup")]
    pub element_group: Option<Box<Group>>,
    /// An image page element.
    pub image: Option<Image>,
    /// A line page element.
    pub line: Option<Line>,
    /// The object ID for this page element. Object IDs used by google.apps.slides.v1.Page and google.apps.slides.v1.PageElement share the same namespace.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// A generic shape.
    pub shape: Option<Shape>,
    /// A linked chart embedded from Google Sheets. Unlinked charts are represented as images.
    #[serde(rename = "sheetsChart")]
    pub sheets_chart: Option<SheetsChart>,
    /// The size of the page element.
    pub size: Option<Size>,
    /// A Speaker Spotlight.
    #[serde(rename = "speakerSpotlight")]
    pub speaker_spotlight: Option<SpeakerSpotlight>,
    /// A table page element.
    pub table: Option<Table>,
    /// The title of the page element. Combined with description to display alt text. The field is not supported for Group elements.
    pub title: Option<String>,
    /// The transform of the page element. The visual appearance of the page element is determined by its absolute transform. To compute the absolute transform, preconcatenate a page element's transform with the transforms of all of its parent groups. If the page element is not in a group, its absolute transform is the same as the value in this field. The initial transform for the newly created Group is always the identity transform.
    pub transform: Option<AffineTransform>,
    /// A video page element.
    pub video: Option<Video>,
    /// A word art page element.
    #[serde(rename = "wordArt")]
    pub word_art: Option<WordArt>,
}

impl common::Part for PageElement {}

/// Common properties for a page element. Note: When you initially create a PageElement, the API may modify the values of both `size` and `transform`, but the visual size will be unchanged.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PageElementProperties {
    /// The object ID of the page where the element is located.
    #[serde(rename = "pageObjectId")]
    pub page_object_id: Option<String>,
    /// The size of the element.
    pub size: Option<Size>,
    /// The transform for the element.
    pub transform: Option<AffineTransform>,
}

impl common::Part for PageElementProperties {}

/// The properties of the Page. The page will inherit properties from the parent page. Depending on the page type the hierarchy is defined in either SlideProperties or LayoutProperties.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PageProperties {
    /// The color scheme of the page. If unset, the color scheme is inherited from a parent page. If the page has no parent, the color scheme uses a default Slides color scheme, matching the defaults in the Slides editor. Only the concrete colors of the first 12 ThemeColorTypes are editable. In addition, only the color scheme on `Master` pages can be updated. To update the field, a color scheme containing mappings from all the first 12 ThemeColorTypes to their concrete colors must be provided. Colors for the remaining ThemeColorTypes will be ignored.
    #[serde(rename = "colorScheme")]
    pub color_scheme: Option<ColorScheme>,
    /// The background fill of the page. If unset, the background fill is inherited from a parent page if it exists. If the page has no parent, then the background fill defaults to the corresponding fill in the Slides editor.
    #[serde(rename = "pageBackgroundFill")]
    pub page_background_fill: Option<PageBackgroundFill>,
}

impl common::Part for PageProperties {}

/// A TextElement kind that represents the beginning of a new paragraph.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParagraphMarker {
    /// The bullet for this paragraph. If not present, the paragraph does not belong to a list.
    pub bullet: Option<Bullet>,
    /// The paragraph's style
    pub style: Option<ParagraphStyle>,
}

impl common::Part for ParagraphMarker {}

/// Styles that apply to a whole paragraph. If this text is contained in a shape with a parent placeholder, then these paragraph styles may be inherited from the parent. Which paragraph styles are inherited depend on the nesting level of lists: * A paragraph not in a list will inherit its paragraph style from the paragraph at the 0 nesting level of the list inside the parent placeholder. * A paragraph in a list will inherit its paragraph style from the paragraph at its corresponding nesting level of the list inside the parent placeholder. Inherited paragraph styles are represented as unset fields in this message.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParagraphStyle {
    /// The text alignment for this paragraph.
    pub alignment: Option<String>,
    /// The text direction of this paragraph. If unset, the value defaults to LEFT_TO_RIGHT since text direction is not inherited.
    pub direction: Option<String>,
    /// The amount indentation for the paragraph on the side that corresponds to the end of the text, based on the current text direction. If unset, the value is inherited from the parent.
    #[serde(rename = "indentEnd")]
    pub indent_end: Option<Dimension>,
    /// The amount of indentation for the start of the first line of the paragraph. If unset, the value is inherited from the parent.
    #[serde(rename = "indentFirstLine")]
    pub indent_first_line: Option<Dimension>,
    /// The amount indentation for the paragraph on the side that corresponds to the start of the text, based on the current text direction. If unset, the value is inherited from the parent.
    #[serde(rename = "indentStart")]
    pub indent_start: Option<Dimension>,
    /// The amount of space between lines, as a percentage of normal, where normal is represented as 100.0. If unset, the value is inherited from the parent.
    #[serde(rename = "lineSpacing")]
    pub line_spacing: Option<f32>,
    /// The amount of extra space above the paragraph. If unset, the value is inherited from the parent.
    #[serde(rename = "spaceAbove")]
    pub space_above: Option<Dimension>,
    /// The amount of extra space below the paragraph. If unset, the value is inherited from the parent.
    #[serde(rename = "spaceBelow")]
    pub space_below: Option<Dimension>,
    /// The spacing mode for the paragraph.
    #[serde(rename = "spacingMode")]
    pub spacing_mode: Option<String>,
}

impl common::Part for ParagraphStyle {}

/// The placeholder information that uniquely identifies a placeholder shape.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Placeholder {
    /// The index of the placeholder. If the same placeholder types are present in the same page, they would have different index values.
    pub index: Option<i32>,
    /// The object ID of this shape's parent placeholder. If unset, the parent placeholder shape does not exist, so the shape does not inherit properties from any other shape.
    #[serde(rename = "parentObjectId")]
    pub parent_object_id: Option<String>,
    /// The type of the placeholder.
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl common::Part for Placeholder {}

/// A Google Slides presentation.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [pages get presentations](PresentationPageGetCall) (none)
/// * [pages get thumbnail presentations](PresentationPageGetThumbnailCall) (none)
/// * [batch update presentations](PresentationBatchUpdateCall) (none)
/// * [create presentations](PresentationCreateCall) (request|response)
/// * [get presentations](PresentationGetCall) (response)
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Presentation {
    /// The layouts in the presentation. A layout is a template that determines how content is arranged and styled on the slides that inherit from that layout.
    pub layouts: Option<Vec<Page>>,
    /// The locale of the presentation, as an IETF BCP 47 language tag.
    pub locale: Option<String>,
    /// The slide masters in the presentation. A slide master contains all common page elements and the common properties for a set of layouts. They serve three purposes: - Placeholder shapes on a master contain the default text styles and shape properties of all placeholder shapes on pages that use that master. - The master page properties define the common page properties inherited by its layouts. - Any other shapes on the master slide appear on all slides using that master, regardless of their layout.
    pub masters: Option<Vec<Page>>,
    /// The notes master in the presentation. It serves three purposes: - Placeholder shapes on a notes master contain the default text styles and shape properties of all placeholder shapes on notes pages. Specifically, a `SLIDE_IMAGE` placeholder shape contains the slide thumbnail, and a `BODY` placeholder shape contains the speaker notes. - The notes master page properties define the common page properties inherited by all notes pages. - Any other shapes on the notes master appear on all notes pages. The notes master is read-only.
    #[serde(rename = "notesMaster")]
    pub notes_master: Option<Page>,
    /// The size of pages in the presentation.
    #[serde(rename = "pageSize")]
    pub page_size: Option<Size>,
    /// The ID of the presentation.
    #[serde(rename = "presentationId")]
    pub presentation_id: Option<String>,
    /// Output only. The revision ID of the presentation. Can be used in update requests to assert the presentation revision hasn't changed since the last read operation. Only populated if the user has edit access to the presentation. The revision ID is not a sequential number but a nebulous string. The format of the revision ID may change over time, so it should be treated opaquely. A returned revision ID is only guaranteed to be valid for 24 hours after it has been returned and cannot be shared across users. If the revision ID is unchanged between calls, then the presentation has not changed. Conversely, a changed ID (for the same presentation and user) usually means the presentation has been updated. However, a changed ID can also be due to internal factors such as ID format changes.
    #[serde(rename = "revisionId")]
    pub revision_id: Option<String>,
    /// The slides in the presentation. A slide inherits properties from a slide layout.
    pub slides: Option<Vec<Page>>,
    /// The title of the presentation.
    pub title: Option<String>,
}

impl common::RequestValue for Presentation {}
impl common::Resource for Presentation {}
impl common::ResponseResult for Presentation {}

/// Specifies a contiguous range of an indexed collection, such as characters in text.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Range {
    /// The optional zero-based index of the end of the collection. Required for `FIXED_RANGE` ranges.
    #[serde(rename = "endIndex")]
    pub end_index: Option<i32>,
    /// The optional zero-based index of the beginning of the collection. Required for `FIXED_RANGE` and `FROM_START_INDEX` ranges.
    #[serde(rename = "startIndex")]
    pub start_index: Option<i32>,
    /// The type of range.
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl common::Part for Range {}

/// A recolor effect applied on an image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Recolor {
    /// The name of the recolor effect. The name is determined from the `recolor_stops` by matching the gradient against the colors in the page's current color scheme. This property is read-only.
    pub name: Option<String>,
    /// The recolor effect is represented by a gradient, which is a list of color stops. The colors in the gradient will replace the corresponding colors at the same position in the color palette and apply to the image. This property is read-only.
    #[serde(rename = "recolorStops")]
    pub recolor_stops: Option<Vec<ColorStop>>,
}

impl common::Part for Recolor {}

/// Refreshes an embedded Google Sheets chart by replacing it with the latest version of the chart from Google Sheets. NOTE: Refreshing charts requires at least one of the spreadsheets.readonly, spreadsheets, drive.readonly, or drive OAuth scopes.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RefreshSheetsChartRequest {
    /// The object ID of the chart to refresh.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for RefreshSheetsChartRequest {}

/// Replaces all shapes that match the given criteria with the provided image. The images replacing the shapes are rectangular after being inserted into the presentation and do not take on the forms of the shapes.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceAllShapesWithImageRequest {
    /// If set, this request will replace all of the shapes that contain the given text.
    #[serde(rename = "containsText")]
    pub contains_text: Option<SubstringMatchCriteria>,
    /// The image replace method. If you specify both a `replace_method` and an `image_replace_method`, the `image_replace_method` takes precedence. If you do not specify a value for `image_replace_method`, but specify a value for `replace_method`, then the specified `replace_method` value is used. If you do not specify either, then CENTER_INSIDE is used.
    #[serde(rename = "imageReplaceMethod")]
    pub image_replace_method: Option<String>,
    /// The image URL. The image is fetched once at insertion time and a copy is stored for display inside the presentation. Images must be less than 50MB in size, cannot exceed 25 megapixels, and must be in one of PNG, JPEG, or GIF format. The provided URL can be at most 2 kB in length. The URL itself is saved with the image, and exposed via the Image.source_url field.
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    /// If non-empty, limits the matches to page elements only on the given pages. Returns a 400 bad request error if given the page object ID of a notes page or a notes master, or if a page with that object ID doesn't exist in the presentation.
    #[serde(rename = "pageObjectIds")]
    pub page_object_ids: Option<Vec<String>>,
    /// The replace method. *Deprecated*: use `image_replace_method` instead. If you specify both a `replace_method` and an `image_replace_method`, the `image_replace_method` takes precedence.
    #[serde(rename = "replaceMethod")]
    pub replace_method: Option<String>,
}

impl common::Part for ReplaceAllShapesWithImageRequest {}

/// The result of replacing shapes with an image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceAllShapesWithImageResponse {
    /// The number of shapes replaced with images.
    #[serde(rename = "occurrencesChanged")]
    pub occurrences_changed: Option<i32>,
}

impl common::Part for ReplaceAllShapesWithImageResponse {}

/// Replaces all shapes that match the given criteria with the provided Google Sheets chart. The chart will be scaled and centered to fit within the bounds of the original shape. NOTE: Replacing shapes with a chart requires at least one of the spreadsheets.readonly, spreadsheets, drive.readonly, or drive OAuth scopes.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceAllShapesWithSheetsChartRequest {
    /// The ID of the specific chart in the Google Sheets spreadsheet.
    #[serde(rename = "chartId")]
    pub chart_id: Option<i32>,
    /// The criteria that the shapes must match in order to be replaced. The request will replace all of the shapes that contain the given text.
    #[serde(rename = "containsText")]
    pub contains_text: Option<SubstringMatchCriteria>,
    /// The mode with which the chart is linked to the source spreadsheet. When not specified, the chart will be an image that is not linked.
    #[serde(rename = "linkingMode")]
    pub linking_mode: Option<String>,
    /// If non-empty, limits the matches to page elements only on the given pages. Returns a 400 bad request error if given the page object ID of a notes page or a notes master, or if a page with that object ID doesn't exist in the presentation.
    #[serde(rename = "pageObjectIds")]
    pub page_object_ids: Option<Vec<String>>,
    /// The ID of the Google Sheets spreadsheet that contains the chart.
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: Option<String>,
}

impl common::Part for ReplaceAllShapesWithSheetsChartRequest {}

/// The result of replacing shapes with a Google Sheets chart.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceAllShapesWithSheetsChartResponse {
    /// The number of shapes replaced with charts.
    #[serde(rename = "occurrencesChanged")]
    pub occurrences_changed: Option<i32>,
}

impl common::Part for ReplaceAllShapesWithSheetsChartResponse {}

/// Replaces all instances of text matching a criteria with replace text.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceAllTextRequest {
    /// Finds text in a shape matching this substring.
    #[serde(rename = "containsText")]
    pub contains_text: Option<SubstringMatchCriteria>,
    /// If non-empty, limits the matches to page elements only on the given pages. Returns a 400 bad request error if given the page object ID of a notes master, or if a page with that object ID doesn't exist in the presentation.
    #[serde(rename = "pageObjectIds")]
    pub page_object_ids: Option<Vec<String>>,
    /// The text that will replace the matched text.
    #[serde(rename = "replaceText")]
    pub replace_text: Option<String>,
}

impl common::Part for ReplaceAllTextRequest {}

/// The result of replacing text.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceAllTextResponse {
    /// The number of occurrences changed by replacing all text.
    #[serde(rename = "occurrencesChanged")]
    pub occurrences_changed: Option<i32>,
}

impl common::Part for ReplaceAllTextResponse {}

/// Replaces an existing image with a new image. Replacing an image removes some image effects from the existing image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplaceImageRequest {
    /// The ID of the existing image that will be replaced. The ID can be retrieved from the response of a get request.
    #[serde(rename = "imageObjectId")]
    pub image_object_id: Option<String>,
    /// The replacement method.
    #[serde(rename = "imageReplaceMethod")]
    pub image_replace_method: Option<String>,
    /// The image URL. The image is fetched once at insertion time and a copy is stored for display inside the presentation. Images must be less than 50MB, cannot exceed 25 megapixels, and must be in PNG, JPEG, or GIF format. The provided URL can't surpass 2 KB in length. The URL is saved with the image, and exposed through the Image.source_url field.
    pub url: Option<String>,
}

impl common::Part for ReplaceImageRequest {}

/// A single kind of update to apply to a presentation.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    /// Creates an image.
    #[serde(rename = "createImage")]
    pub create_image: Option<CreateImageRequest>,
    /// Creates a line.
    #[serde(rename = "createLine")]
    pub create_line: Option<CreateLineRequest>,
    /// Creates bullets for paragraphs.
    #[serde(rename = "createParagraphBullets")]
    pub create_paragraph_bullets: Option<CreateParagraphBulletsRequest>,
    /// Creates a new shape.
    #[serde(rename = "createShape")]
    pub create_shape: Option<CreateShapeRequest>,
    /// Creates an embedded Google Sheets chart.
    #[serde(rename = "createSheetsChart")]
    pub create_sheets_chart: Option<CreateSheetsChartRequest>,
    /// Creates a new slide.
    #[serde(rename = "createSlide")]
    pub create_slide: Option<CreateSlideRequest>,
    /// Creates a new table.
    #[serde(rename = "createTable")]
    pub create_table: Option<CreateTableRequest>,
    /// Creates a video.
    #[serde(rename = "createVideo")]
    pub create_video: Option<CreateVideoRequest>,
    /// Deletes a page or page element from the presentation.
    #[serde(rename = "deleteObject")]
    pub delete_object: Option<DeleteObjectRequest>,
    /// Deletes bullets from paragraphs.
    #[serde(rename = "deleteParagraphBullets")]
    pub delete_paragraph_bullets: Option<DeleteParagraphBulletsRequest>,
    /// Deletes a column from a table.
    #[serde(rename = "deleteTableColumn")]
    pub delete_table_column: Option<DeleteTableColumnRequest>,
    /// Deletes a row from a table.
    #[serde(rename = "deleteTableRow")]
    pub delete_table_row: Option<DeleteTableRowRequest>,
    /// Deletes text from a shape or a table cell.
    #[serde(rename = "deleteText")]
    pub delete_text: Option<DeleteTextRequest>,
    /// Duplicates a slide or page element.
    #[serde(rename = "duplicateObject")]
    pub duplicate_object: Option<DuplicateObjectRequest>,
    /// Groups objects, such as page elements.
    #[serde(rename = "groupObjects")]
    pub group_objects: Option<GroupObjectsRequest>,
    /// Inserts columns into a table.
    #[serde(rename = "insertTableColumns")]
    pub insert_table_columns: Option<InsertTableColumnsRequest>,
    /// Inserts rows into a table.
    #[serde(rename = "insertTableRows")]
    pub insert_table_rows: Option<InsertTableRowsRequest>,
    /// Inserts text into a shape or table cell.
    #[serde(rename = "insertText")]
    pub insert_text: Option<InsertTextRequest>,
    /// Merges cells in a Table.
    #[serde(rename = "mergeTableCells")]
    pub merge_table_cells: Option<MergeTableCellsRequest>,
    /// Refreshes a Google Sheets chart.
    #[serde(rename = "refreshSheetsChart")]
    pub refresh_sheets_chart: Option<RefreshSheetsChartRequest>,
    /// Replaces all shapes matching some criteria with an image.
    #[serde(rename = "replaceAllShapesWithImage")]
    pub replace_all_shapes_with_image: Option<ReplaceAllShapesWithImageRequest>,
    /// Replaces all shapes matching some criteria with a Google Sheets chart.
    #[serde(rename = "replaceAllShapesWithSheetsChart")]
    pub replace_all_shapes_with_sheets_chart: Option<ReplaceAllShapesWithSheetsChartRequest>,
    /// Replaces all instances of specified text.
    #[serde(rename = "replaceAllText")]
    pub replace_all_text: Option<ReplaceAllTextRequest>,
    /// Replaces an existing image with a new image.
    #[serde(rename = "replaceImage")]
    pub replace_image: Option<ReplaceImageRequest>,
    /// Reroutes a line such that it's connected at the two closest connection sites on the connected page elements.
    #[serde(rename = "rerouteLine")]
    pub reroute_line: Option<RerouteLineRequest>,
    /// Ungroups objects, such as groups.
    #[serde(rename = "ungroupObjects")]
    pub ungroup_objects: Option<UngroupObjectsRequest>,
    /// Unmerges cells in a Table.
    #[serde(rename = "unmergeTableCells")]
    pub unmerge_table_cells: Option<UnmergeTableCellsRequest>,
    /// Updates the properties of an Image.
    #[serde(rename = "updateImageProperties")]
    pub update_image_properties: Option<UpdateImagePropertiesRequest>,
    /// Updates the category of a line.
    #[serde(rename = "updateLineCategory")]
    pub update_line_category: Option<UpdateLineCategoryRequest>,
    /// Updates the properties of a Line.
    #[serde(rename = "updateLineProperties")]
    pub update_line_properties: Option<UpdateLinePropertiesRequest>,
    /// Updates the alt text title and/or description of a page element.
    #[serde(rename = "updatePageElementAltText")]
    pub update_page_element_alt_text: Option<UpdatePageElementAltTextRequest>,
    /// Updates the transform of a page element.
    #[serde(rename = "updatePageElementTransform")]
    pub update_page_element_transform: Option<UpdatePageElementTransformRequest>,
    /// Updates the Z-order of page elements.
    #[serde(rename = "updatePageElementsZOrder")]
    pub update_page_elements_z_order: Option<UpdatePageElementsZOrderRequest>,
    /// Updates the properties of a Page.
    #[serde(rename = "updatePageProperties")]
    pub update_page_properties: Option<UpdatePagePropertiesRequest>,
    /// Updates the styling of paragraphs within a Shape or Table.
    #[serde(rename = "updateParagraphStyle")]
    pub update_paragraph_style: Option<UpdateParagraphStyleRequest>,
    /// Updates the properties of a Shape.
    #[serde(rename = "updateShapeProperties")]
    pub update_shape_properties: Option<UpdateShapePropertiesRequest>,
    /// Updates the properties of a Slide
    #[serde(rename = "updateSlideProperties")]
    pub update_slide_properties: Option<UpdateSlidePropertiesRequest>,
    /// Updates the position of a set of slides in the presentation.
    #[serde(rename = "updateSlidesPosition")]
    pub update_slides_position: Option<UpdateSlidesPositionRequest>,
    /// Updates the properties of the table borders in a Table.
    #[serde(rename = "updateTableBorderProperties")]
    pub update_table_border_properties: Option<UpdateTableBorderPropertiesRequest>,
    /// Updates the properties of a TableCell.
    #[serde(rename = "updateTableCellProperties")]
    pub update_table_cell_properties: Option<UpdateTableCellPropertiesRequest>,
    /// Updates the properties of a Table column.
    #[serde(rename = "updateTableColumnProperties")]
    pub update_table_column_properties: Option<UpdateTableColumnPropertiesRequest>,
    /// Updates the properties of a Table row.
    #[serde(rename = "updateTableRowProperties")]
    pub update_table_row_properties: Option<UpdateTableRowPropertiesRequest>,
    /// Updates the styling of text within a Shape or Table.
    #[serde(rename = "updateTextStyle")]
    pub update_text_style: Option<UpdateTextStyleRequest>,
    /// Updates the properties of a Video.
    #[serde(rename = "updateVideoProperties")]
    pub update_video_properties: Option<UpdateVideoPropertiesRequest>,
}

impl common::Part for Request {}

/// Reroutes a line such that it's connected at the two closest connection sites on the connected page elements.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RerouteLineRequest {
    /// The object ID of the line to reroute. Only a line with a category indicating it is a "connector" can be rerouted. The start and end connections of the line must be on different page elements.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for RerouteLineRequest {}

/// A single response from an update.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Response {
    /// The result of creating an image.
    #[serde(rename = "createImage")]
    pub create_image: Option<CreateImageResponse>,
    /// The result of creating a line.
    #[serde(rename = "createLine")]
    pub create_line: Option<CreateLineResponse>,
    /// The result of creating a shape.
    #[serde(rename = "createShape")]
    pub create_shape: Option<CreateShapeResponse>,
    /// The result of creating a Google Sheets chart.
    #[serde(rename = "createSheetsChart")]
    pub create_sheets_chart: Option<CreateSheetsChartResponse>,
    /// The result of creating a slide.
    #[serde(rename = "createSlide")]
    pub create_slide: Option<CreateSlideResponse>,
    /// The result of creating a table.
    #[serde(rename = "createTable")]
    pub create_table: Option<CreateTableResponse>,
    /// The result of creating a video.
    #[serde(rename = "createVideo")]
    pub create_video: Option<CreateVideoResponse>,
    /// The result of duplicating an object.
    #[serde(rename = "duplicateObject")]
    pub duplicate_object: Option<DuplicateObjectResponse>,
    /// The result of grouping objects.
    #[serde(rename = "groupObjects")]
    pub group_objects: Option<GroupObjectsResponse>,
    /// The result of replacing all shapes matching some criteria with an image.
    #[serde(rename = "replaceAllShapesWithImage")]
    pub replace_all_shapes_with_image: Option<ReplaceAllShapesWithImageResponse>,
    /// The result of replacing all shapes matching some criteria with a Google Sheets chart.
    #[serde(rename = "replaceAllShapesWithSheetsChart")]
    pub replace_all_shapes_with_sheets_chart: Option<ReplaceAllShapesWithSheetsChartResponse>,
    /// The result of replacing text.
    #[serde(rename = "replaceAllText")]
    pub replace_all_text: Option<ReplaceAllTextResponse>,
}

impl common::Part for Response {}

/// An RGB color.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RgbColor {
    /// The blue component of the color, from 0.0 to 1.0.
    pub blue: Option<f32>,
    /// The green component of the color, from 0.0 to 1.0.
    pub green: Option<f32>,
    /// The red component of the color, from 0.0 to 1.0.
    pub red: Option<f32>,
}

impl common::Part for RgbColor {}

/// The shadow properties of a page element. If these fields are unset, they may be inherited from a parent placeholder if it exists. If there is no parent, the fields will default to the value used for new page elements created in the Slides editor, which may depend on the page element kind.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Shadow {
    /// The alignment point of the shadow, that sets the origin for translate, scale and skew of the shadow. This property is read-only.
    pub alignment: Option<String>,
    /// The alpha of the shadow's color, from 0.0 to 1.0.
    pub alpha: Option<f32>,
    /// The radius of the shadow blur. The larger the radius, the more diffuse the shadow becomes.
    #[serde(rename = "blurRadius")]
    pub blur_radius: Option<Dimension>,
    /// The shadow color value.
    pub color: Option<OpaqueColor>,
    /// The shadow property state. Updating the shadow on a page element will implicitly update this field to `RENDERED`, unless another value is specified in the same request. To have no shadow on a page element, set this field to `NOT_RENDERED`. In this case, any other shadow fields set in the same request will be ignored.
    #[serde(rename = "propertyState")]
    pub property_state: Option<String>,
    /// Whether the shadow should rotate with the shape. This property is read-only.
    #[serde(rename = "rotateWithShape")]
    pub rotate_with_shape: Option<bool>,
    /// Transform that encodes the translate, scale, and skew of the shadow, relative to the alignment position.
    pub transform: Option<AffineTransform>,
    /// The type of the shadow. This property is read-only.
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl common::Part for Shadow {}

/// A PageElement kind representing a generic shape that doesn't have a more specific classification. For more information, see [Size and position page elements](https://developers.google.com/workspace/slides/api/guides/transform).
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Shape {
    /// Placeholders are page elements that inherit from corresponding placeholders on layouts and masters. If set, the shape is a placeholder shape and any inherited properties can be resolved by looking at the parent placeholder identified by the Placeholder.parent_object_id field.
    pub placeholder: Option<Placeholder>,
    /// The properties of the shape.
    #[serde(rename = "shapeProperties")]
    pub shape_properties: Option<ShapeProperties>,
    /// The type of the shape.
    #[serde(rename = "shapeType")]
    pub shape_type: Option<String>,
    /// The text content of the shape.
    pub text: Option<TextContent>,
}

impl common::Part for Shape {}

/// The shape background fill.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShapeBackgroundFill {
    /// The background fill property state. Updating the fill on a shape will implicitly update this field to `RENDERED`, unless another value is specified in the same request. To have no fill on a shape, set this field to `NOT_RENDERED`. In this case, any other fill fields set in the same request will be ignored.
    #[serde(rename = "propertyState")]
    pub property_state: Option<String>,
    /// Solid color fill.
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<SolidFill>,
}

impl common::Part for ShapeBackgroundFill {}

/// The properties of a Shape. If the shape is a placeholder shape as determined by the placeholder field, then these properties may be inherited from a parent placeholder shape. Determining the rendered value of the property depends on the corresponding property_state field value. Any text autofit settings on the shape are automatically deactivated by requests that can impact how text fits in the shape.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShapeProperties {
    /// The autofit properties of the shape. This property is only set for shapes that allow text.
    pub autofit: Option<Autofit>,
    /// The alignment of the content in the shape. If unspecified, the alignment is inherited from a parent placeholder if it exists. If the shape has no parent, the default alignment matches the alignment for new shapes created in the Slides editor.
    #[serde(rename = "contentAlignment")]
    pub content_alignment: Option<String>,
    /// The hyperlink destination of the shape. If unset, there is no link. Links are not inherited from parent placeholders.
    pub link: Option<Link>,
    /// The outline of the shape. If unset, the outline is inherited from a parent placeholder if it exists. If the shape has no parent, then the default outline depends on the shape type, matching the defaults for new shapes created in the Slides editor.
    pub outline: Option<Outline>,
    /// The shadow properties of the shape. If unset, the shadow is inherited from a parent placeholder if it exists. If the shape has no parent, then the default shadow matches the defaults for new shapes created in the Slides editor. This property is read-only.
    pub shadow: Option<Shadow>,
    /// The background fill of the shape. If unset, the background fill is inherited from a parent placeholder if it exists. If the shape has no parent, then the default background fill depends on the shape type, matching the defaults for new shapes created in the Slides editor.
    #[serde(rename = "shapeBackgroundFill")]
    pub shape_background_fill: Option<ShapeBackgroundFill>,
}

impl common::Part for ShapeProperties {}

/// A PageElement kind representing a linked chart embedded from Google Sheets.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SheetsChart {
    /// The ID of the specific chart in the Google Sheets spreadsheet that is embedded.
    #[serde(rename = "chartId")]
    pub chart_id: Option<i32>,
    /// The URL of an image of the embedded chart, with a default lifetime of 30 minutes. This URL is tagged with the account of the requester. Anyone with the URL effectively accesses the image as the original requester. Access to the image may be lost if the presentation's sharing settings change.
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
    /// The properties of the Sheets chart.
    #[serde(rename = "sheetsChartProperties")]
    pub sheets_chart_properties: Option<SheetsChartProperties>,
    /// The ID of the Google Sheets spreadsheet that contains the source chart.
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: Option<String>,
}

impl common::Part for SheetsChart {}

/// The properties of the SheetsChart.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SheetsChartProperties {
    /// The properties of the embedded chart image.
    #[serde(rename = "chartImageProperties")]
    pub chart_image_properties: Option<ImageProperties>,
}

impl common::Part for SheetsChartProperties {}

/// A width and height.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Size {
    /// The height of the object.
    pub height: Option<Dimension>,
    /// The width of the object.
    pub width: Option<Dimension>,
}

impl common::Part for Size {}

/// The properties of Page that are only relevant for pages with page_type SLIDE.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SlideProperties {
    /// Whether the slide is skipped in the presentation mode. Defaults to false.
    #[serde(rename = "isSkipped")]
    pub is_skipped: Option<bool>,
    /// The object ID of the layout that this slide is based on. This property is read-only.
    #[serde(rename = "layoutObjectId")]
    pub layout_object_id: Option<String>,
    /// The object ID of the master that this slide is based on. This property is read-only.
    #[serde(rename = "masterObjectId")]
    pub master_object_id: Option<String>,
    /// The notes page that this slide is associated with. It defines the visual appearance of a notes page when printing or exporting slides with speaker notes. A notes page inherits properties from the notes master. The placeholder shape with type BODY on the notes page contains the speaker notes for this slide. The ID of this shape is identified by the speakerNotesObjectId field. The notes page is read-only except for the text content and styles of the speaker notes shape. This property is read-only.
    #[serde(rename = "notesPage")]
    pub notes_page: Option<Box<Page>>,
}

impl common::Part for SlideProperties {}

/// A solid color fill. The page or page element is filled entirely with the specified color value. If any field is unset, its value may be inherited from a parent placeholder if it exists.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SolidFill {
    /// The fraction of this `color` that should be applied to the pixel. That is, the final pixel color is defined by the equation: pixel color = alpha * (color) + (1.0 - alpha) * (background color) This means that a value of 1.0 corresponds to a solid color, whereas a value of 0.0 corresponds to a completely transparent color.
    pub alpha: Option<f32>,
    /// The color value of the solid fill.
    pub color: Option<OpaqueColor>,
}

impl common::Part for SolidFill {}

/// A PageElement kind representing a Speaker Spotlight.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpeakerSpotlight {
    /// The properties of the Speaker Spotlight.
    #[serde(rename = "speakerSpotlightProperties")]
    pub speaker_spotlight_properties: Option<SpeakerSpotlightProperties>,
}

impl common::Part for SpeakerSpotlight {}

/// The properties of the SpeakerSpotlight.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpeakerSpotlightProperties {
    /// The outline of the Speaker Spotlight. If not set, it has no outline.
    pub outline: Option<Outline>,
    /// The shadow of the Speaker Spotlight. If not set, it has no shadow.
    pub shadow: Option<Shadow>,
}

impl common::Part for SpeakerSpotlightProperties {}

/// The stretched picture fill. The page or page element is filled entirely with the specified picture. The picture is stretched to fit its container.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StretchedPictureFill {
    /// Reading the content_url: An URL to a picture with a default lifetime of 30 minutes. This URL is tagged with the account of the requester. Anyone with the URL effectively accesses the picture as the original requester. Access to the picture may be lost if the presentation's sharing settings change. Writing the content_url: The picture is fetched once at insertion time and a copy is stored for display inside the presentation. Pictures must be less than 50MB in size, cannot exceed 25 megapixels, and must be in one of PNG, JPEG, or GIF format. The provided URL can be at most 2 kB in length.
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
    /// The original size of the picture fill. This field is read-only.
    pub size: Option<Size>,
}

impl common::Part for StretchedPictureFill {}

/// A criteria that matches a specific string of text in a shape or table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SubstringMatchCriteria {
    /// Indicates whether the search should respect case: - `True`: the search is case sensitive. - `False`: the search is case insensitive.
    #[serde(rename = "matchCase")]
    pub match_case: Option<bool>,
    /// Optional. True if the find value should be treated as a regular expression. Any backslashes in the pattern should be escaped. - `True`: the search text is treated as a regular expressions. - `False`: the search text is treated as a substring for matching.
    #[serde(rename = "searchByRegex")]
    pub search_by_regex: Option<bool>,
    /// The text to search for in the shape or table.
    pub text: Option<String>,
}

impl common::Part for SubstringMatchCriteria {}

/// A PageElement kind representing a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Table {
    /// Number of columns in the table.
    pub columns: Option<i32>,
    /// Properties of horizontal cell borders. A table's horizontal cell borders are represented as a grid. The grid has one more row than the number of rows in the table and the same number of columns as the table. For example, if the table is 3 x 3, its horizontal borders will be represented as a grid with 4 rows and 3 columns.
    #[serde(rename = "horizontalBorderRows")]
    pub horizontal_border_rows: Option<Vec<TableBorderRow>>,
    /// Number of rows in the table.
    pub rows: Option<i32>,
    /// Properties of each column.
    #[serde(rename = "tableColumns")]
    pub table_columns: Option<Vec<TableColumnProperties>>,
    /// Properties and contents of each row. Cells that span multiple rows are contained in only one of these rows and have a row_span greater than 1.
    #[serde(rename = "tableRows")]
    pub table_rows: Option<Vec<TableRow>>,
    /// Properties of vertical cell borders. A table's vertical cell borders are represented as a grid. The grid has the same number of rows as the table and one more column than the number of columns in the table. For example, if the table is 3 x 3, its vertical borders will be represented as a grid with 3 rows and 4 columns.
    #[serde(rename = "verticalBorderRows")]
    pub vertical_border_rows: Option<Vec<TableBorderRow>>,
}

impl common::Part for Table {}

/// The properties of each border cell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableBorderCell {
    /// The location of the border within the border table.
    pub location: Option<TableCellLocation>,
    /// The border properties.
    #[serde(rename = "tableBorderProperties")]
    pub table_border_properties: Option<TableBorderProperties>,
}

impl common::Part for TableBorderCell {}

/// The fill of the border.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableBorderFill {
    /// Solid fill.
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<SolidFill>,
}

impl common::Part for TableBorderFill {}

/// The border styling properties of the TableBorderCell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableBorderProperties {
    /// The dash style of the border.
    #[serde(rename = "dashStyle")]
    pub dash_style: Option<String>,
    /// The fill of the table border.
    #[serde(rename = "tableBorderFill")]
    pub table_border_fill: Option<TableBorderFill>,
    /// The thickness of the border.
    pub weight: Option<Dimension>,
}

impl common::Part for TableBorderProperties {}

/// Contents of each border row in a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableBorderRow {
    /// Properties of each border cell. When a border's adjacent table cells are merged, it is not included in the response.
    #[serde(rename = "tableBorderCells")]
    pub table_border_cells: Option<Vec<TableBorderCell>>,
}

impl common::Part for TableBorderRow {}

/// Properties and contents of each table cell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableCell {
    /// Column span of the cell.
    #[serde(rename = "columnSpan")]
    pub column_span: Option<i32>,
    /// The location of the cell within the table.
    pub location: Option<TableCellLocation>,
    /// Row span of the cell.
    #[serde(rename = "rowSpan")]
    pub row_span: Option<i32>,
    /// The properties of the table cell.
    #[serde(rename = "tableCellProperties")]
    pub table_cell_properties: Option<TableCellProperties>,
    /// The text content of the cell.
    pub text: Option<TextContent>,
}

impl common::Part for TableCell {}

/// The table cell background fill.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableCellBackgroundFill {
    /// The background fill property state. Updating the fill on a table cell will implicitly update this field to `RENDERED`, unless another value is specified in the same request. To have no fill on a table cell, set this field to `NOT_RENDERED`. In this case, any other fill fields set in the same request will be ignored.
    #[serde(rename = "propertyState")]
    pub property_state: Option<String>,
    /// Solid color fill.
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<SolidFill>,
}

impl common::Part for TableCellBackgroundFill {}

/// A location of a single table cell within a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableCellLocation {
    /// The 0-based column index.
    #[serde(rename = "columnIndex")]
    pub column_index: Option<i32>,
    /// The 0-based row index.
    #[serde(rename = "rowIndex")]
    pub row_index: Option<i32>,
}

impl common::Part for TableCellLocation {}

/// The properties of the TableCell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableCellProperties {
    /// The alignment of the content in the table cell. The default alignment matches the alignment for newly created table cells in the Slides editor.
    #[serde(rename = "contentAlignment")]
    pub content_alignment: Option<String>,
    /// The background fill of the table cell. The default fill matches the fill for newly created table cells in the Slides editor.
    #[serde(rename = "tableCellBackgroundFill")]
    pub table_cell_background_fill: Option<TableCellBackgroundFill>,
}

impl common::Part for TableCellProperties {}

/// Properties of each column in a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableColumnProperties {
    /// Width of a column.
    #[serde(rename = "columnWidth")]
    pub column_width: Option<Dimension>,
}

impl common::Part for TableColumnProperties {}

/// A table range represents a reference to a subset of a table. It's important to note that the cells specified by a table range do not necessarily form a rectangle. For example, let's say we have a 3 x 3 table where all the cells of the last row are merged together. The table looks like this: [ ] A table range with location = (0, 0), row span = 3 and column span = 2 specifies the following cells: x x [ x x x ]
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableRange {
    /// The column span of the table range.
    #[serde(rename = "columnSpan")]
    pub column_span: Option<i32>,
    /// The starting location of the table range.
    pub location: Option<TableCellLocation>,
    /// The row span of the table range.
    #[serde(rename = "rowSpan")]
    pub row_span: Option<i32>,
}

impl common::Part for TableRange {}

/// Properties and contents of each row in a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableRow {
    /// Height of a row.
    #[serde(rename = "rowHeight")]
    pub row_height: Option<Dimension>,
    /// Properties and contents of each cell. Cells that span multiple columns are represented only once with a column_span greater than 1. As a result, the length of this collection does not always match the number of columns of the entire table.
    #[serde(rename = "tableCells")]
    pub table_cells: Option<Vec<TableCell>>,
    /// Properties of the row.
    #[serde(rename = "tableRowProperties")]
    pub table_row_properties: Option<TableRowProperties>,
}

impl common::Part for TableRow {}

/// Properties of each row in a table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableRowProperties {
    /// Minimum height of the row. The row will be rendered in the Slides editor at a height equal to or greater than this value in order to show all the text in the row's cell(s).
    #[serde(rename = "minRowHeight")]
    pub min_row_height: Option<Dimension>,
}

impl common::Part for TableRowProperties {}

/// The general text content. The text must reside in a compatible shape (e.g. text box or rectangle) or a table cell in a page.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextContent {
    /// The bulleted lists contained in this text, keyed by list ID.
    pub lists: Option<HashMap<String, List>>,
    /// The text contents broken down into its component parts, including styling information. This property is read-only.
    #[serde(rename = "textElements")]
    pub text_elements: Option<Vec<TextElement>>,
}

impl common::Part for TextContent {}

/// A TextElement describes the content of a range of indices in the text content of a Shape or TableCell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextElement {
    /// A TextElement representing a spot in the text that is dynamically replaced with content that can change over time.
    #[serde(rename = "autoText")]
    pub auto_text: Option<AutoText>,
    /// The zero-based end index of this text element, exclusive, in Unicode code units.
    #[serde(rename = "endIndex")]
    pub end_index: Option<i32>,
    /// A marker representing the beginning of a new paragraph. The `start_index` and `end_index` of this TextElement represent the range of the paragraph. Other TextElements with an index range contained inside this paragraph's range are considered to be part of this paragraph. The range of indices of two separate paragraphs will never overlap.
    #[serde(rename = "paragraphMarker")]
    pub paragraph_marker: Option<ParagraphMarker>,
    /// The zero-based start index of this text element, in Unicode code units.
    #[serde(rename = "startIndex")]
    pub start_index: Option<i32>,
    /// A TextElement representing a run of text where all of the characters in the run have the same TextStyle. The `start_index` and `end_index` of TextRuns will always be fully contained in the index range of a single `paragraph_marker` TextElement. In other words, a TextRun will never span multiple paragraphs.
    #[serde(rename = "textRun")]
    pub text_run: Option<TextRun>,
}

impl common::Part for TextElement {}

/// A TextElement kind that represents a run of text that all has the same styling.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextRun {
    /// The text of this run.
    pub content: Option<String>,
    /// The styling applied to this run.
    pub style: Option<TextStyle>,
}

impl common::Part for TextRun {}

/// Represents the styling that can be applied to a TextRun. If this text is contained in a shape with a parent placeholder, then these text styles may be inherited from the parent. Which text styles are inherited depend on the nesting level of lists: * A text run in a paragraph that is not in a list will inherit its text style from the the newline character in the paragraph at the 0 nesting level of the list inside the parent placeholder. * A text run in a paragraph that is in a list will inherit its text style from the newline character in the paragraph at its corresponding nesting level of the list inside the parent placeholder. Inherited text styles are represented as unset fields in this message. If text is contained in a shape without a parent placeholder, unsetting these fields will revert the style to a value matching the defaults in the Slides editor.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextStyle {
    /// The background color of the text. If set, the color is either opaque or transparent, depending on if the `opaque_color` field in it is set.
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<OptionalColor>,
    /// The text's vertical offset from its normal position. Text with `SUPERSCRIPT` or `SUBSCRIPT` baseline offsets is automatically rendered in a smaller font size, computed based on the `font_size` field. The `font_size` itself is not affected by changes in this field.
    #[serde(rename = "baselineOffset")]
    pub baseline_offset: Option<String>,
    /// Whether or not the text is rendered as bold.
    pub bold: Option<bool>,
    /// The font family of the text. The font family can be any font from the Font menu in Slides or from [Google Fonts] (https://fonts.google.com/). If the font name is unrecognized, the text is rendered in `Arial`. Some fonts can affect the weight of the text. If an update request specifies values for both `font_family` and `bold`, the explicitly-set `bold` value is used.
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,
    /// The size of the text's font. When read, the `font_size` will specified in points.
    #[serde(rename = "fontSize")]
    pub font_size: Option<Dimension>,
    /// The color of the text itself. If set, the color is either opaque or transparent, depending on if the `opaque_color` field in it is set.
    #[serde(rename = "foregroundColor")]
    pub foreground_color: Option<OptionalColor>,
    /// Whether or not the text is italicized.
    pub italic: Option<bool>,
    /// The hyperlink destination of the text. If unset, there is no link. Links are not inherited from parent text. Changing the link in an update request causes some other changes to the text style of the range: * When setting a link, the text foreground color will be set to ThemeColorType.HYPERLINK and the text will be underlined. If these fields are modified in the same request, those values will be used instead of the link defaults. * Setting a link on a text range that overlaps with an existing link will also update the existing link to point to the new URL. * Links are not settable on newline characters. As a result, setting a link on a text range that crosses a paragraph boundary, such as `"ABC\n123"`, will separate the newline character(s) into their own text runs. The link will be applied separately to the runs before and after the newline. * Removing a link will update the text style of the range to match the style of the preceding text (or the default text styles if the preceding text is another link) unless different styles are being set in the same request.
    pub link: Option<Link>,
    /// Whether or not the text is in small capital letters.
    #[serde(rename = "smallCaps")]
    pub small_caps: Option<bool>,
    /// Whether or not the text is struck through.
    pub strikethrough: Option<bool>,
    /// Whether or not the text is underlined.
    pub underline: Option<bool>,
    /// The font family and rendered weight of the text. This field is an extension of `font_family` meant to support explicit font weights without breaking backwards compatibility. As such, when reading the style of a range of text, the value of `weighted_font_family#font_family` will always be equal to that of `font_family`. However, when writing, if both fields are included in the field mask (either explicitly or through the wildcard `"*"`), their values are reconciled as follows: * If `font_family` is set and `weighted_font_family` is not, the value of `font_family` is applied with weight `400` ("normal"). * If both fields are set, the value of `font_family` must match that of `weighted_font_family#font_family`. If so, the font family and weight of `weighted_font_family` is applied. Otherwise, a 400 bad request error is returned. * If `weighted_font_family` is set and `font_family` is not, the font family and weight of `weighted_font_family` is applied. * If neither field is set, the font family and weight of the text inherit from the parent. Note that these properties cannot inherit separately from each other. If an update request specifies values for both `weighted_font_family` and `bold`, the `weighted_font_family` is applied first, then `bold`. If `weighted_font_family#weight` is not set, it defaults to `400`. If `weighted_font_family` is set, then `weighted_font_family#font_family` must also be set with a non-empty value. Otherwise, a 400 bad request error is returned.
    #[serde(rename = "weightedFontFamily")]
    pub weighted_font_family: Option<WeightedFontFamily>,
}

impl common::Part for TextStyle {}

/// A pair mapping a theme color type to the concrete color it represents.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeColorPair {
    /// The concrete color corresponding to the theme color type above.
    pub color: Option<RgbColor>,
    /// The type of the theme color.
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl common::Part for ThemeColorPair {}

/// The thumbnail of a page.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [pages get thumbnail presentations](PresentationPageGetThumbnailCall) (response)
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Thumbnail {
    /// The content URL of the thumbnail image. The URL to the image has a default lifetime of 30 minutes. This URL is tagged with the account of the requester. Anyone with the URL effectively accesses the image as the original requester. Access to the image may be lost if the presentation's sharing settings change. The mime type of the thumbnail image is the same as specified in the `GetPageThumbnailRequest`.
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
    /// The positive height in pixels of the thumbnail image.
    pub height: Option<i32>,
    /// The positive width in pixels of the thumbnail image.
    pub width: Option<i32>,
}

impl common::ResponseResult for Thumbnail {}

/// Ungroups objects, such as groups.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UngroupObjectsRequest {
    /// The object IDs of the objects to ungroup. Only groups that are not inside other groups can be ungrouped. All the groups should be on the same page. The group itself is deleted. The visual sizes and positions of all the children are preserved.
    #[serde(rename = "objectIds")]
    pub object_ids: Option<Vec<String>>,
}

impl common::Part for UngroupObjectsRequest {}

/// Unmerges cells in a Table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UnmergeTableCellsRequest {
    /// The object ID of the table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The table range specifying which cells of the table to unmerge. All merged cells in this range will be unmerged, and cells that are already unmerged will not be affected. If the range has no merged cells, the request will do nothing. If there is text in any of the merged cells, the text will remain in the upper-left ("head") cell of the resulting block of unmerged cells.
    #[serde(rename = "tableRange")]
    pub table_range: Option<TableRange>,
}

impl common::Part for UnmergeTableCellsRequest {}

/// Update the properties of an Image.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateImagePropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `imageProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the image outline color, set `fields` to `"outline.outlineFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The image properties to update.
    #[serde(rename = "imageProperties")]
    pub image_properties: Option<ImageProperties>,
    /// The object ID of the image the updates are applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for UpdateImagePropertiesRequest {}

/// Updates the category of a line.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateLineCategoryRequest {
    /// The line category to update to. The exact line type is determined based on the category to update to and how it's routed to connect to other page elements.
    #[serde(rename = "lineCategory")]
    pub line_category: Option<String>,
    /// The object ID of the line the update is applied to. Only a line with a category indicating it is a "connector" can be updated. The line may be rerouted after updating its category.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for UpdateLineCategoryRequest {}

/// Updates the properties of a Line.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateLinePropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `lineProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the line solid fill color, set `fields` to `"lineFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The line properties to update.
    #[serde(rename = "lineProperties")]
    pub line_properties: Option<LineProperties>,
    /// The object ID of the line the update is applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

impl common::Part for UpdateLinePropertiesRequest {}

/// Updates the alt text title and/or description of a page element.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdatePageElementAltTextRequest {
    /// The updated alt text description of the page element. If unset the existing value will be maintained. The description is exposed to screen readers and other accessibility interfaces. Only use human readable values related to the content of the page element.
    pub description: Option<String>,
    /// The object ID of the page element the updates are applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The updated alt text title of the page element. If unset the existing value will be maintained. The title is exposed to screen readers and other accessibility interfaces. Only use human readable values related to the content of the page element.
    pub title: Option<String>,
}

impl common::Part for UpdatePageElementAltTextRequest {}

/// Updates the transform of a page element. Updating the transform of a group will change the absolute transform of the page elements in that group, which can change their visual appearance. See the documentation for PageElement.transform for more details.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdatePageElementTransformRequest {
    /// The apply mode of the transform update.
    #[serde(rename = "applyMode")]
    pub apply_mode: Option<String>,
    /// The object ID of the page element to update.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The input transform matrix used to update the page element.
    pub transform: Option<AffineTransform>,
}

impl common::Part for UpdatePageElementTransformRequest {}

/// Updates the Z-order of page elements. Z-order is an ordering of the elements on the page from back to front. The page element in the front may cover the elements that are behind it.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdatePageElementsZOrderRequest {
    /// The Z-order operation to apply on the page elements. When applying the operation on multiple page elements, the relative Z-orders within these page elements before the operation is maintained.
    pub operation: Option<String>,
    /// The object IDs of the page elements to update. All the page elements must be on the same page and must not be grouped.
    #[serde(rename = "pageElementObjectIds")]
    pub page_element_object_ids: Option<Vec<String>>,
}

impl common::Part for UpdatePageElementsZOrderRequest {}

/// Updates the properties of a Page.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdatePagePropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `pageProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the page background solid fill color, set `fields` to `"pageBackgroundFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the page the update is applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The page properties to update.
    #[serde(rename = "pageProperties")]
    pub page_properties: Option<PageProperties>,
}

impl common::Part for UpdatePagePropertiesRequest {}

/// Updates the styling for all of the paragraphs within a Shape or Table that overlap with the given text index range.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateParagraphStyleRequest {
    /// The location of the cell in the table containing the paragraph(s) to style. If `object_id` refers to a table, `cell_location` must have a value. Otherwise, it must not.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The fields that should be updated. At least one field must be specified. The root `style` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example, to update the paragraph alignment, set `fields` to `"alignment"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the shape or table with the text to be styled.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The paragraph's style.
    pub style: Option<ParagraphStyle>,
    /// The range of text containing the paragraph(s) to style.
    #[serde(rename = "textRange")]
    pub text_range: Option<Range>,
}

impl common::Part for UpdateParagraphStyleRequest {}

/// Update the properties of a Shape.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateShapePropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `shapeProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the shape background solid fill color, set `fields` to `"shapeBackgroundFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the shape the updates are applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The shape properties to update.
    #[serde(rename = "shapeProperties")]
    pub shape_properties: Option<ShapeProperties>,
}

impl common::Part for UpdateShapePropertiesRequest {}

/// Updates the properties of a Slide.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateSlidePropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root 'slideProperties' is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update whether a slide is skipped, set `fields` to `"isSkipped"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the slide the update is applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The slide properties to update.
    #[serde(rename = "slideProperties")]
    pub slide_properties: Option<SlideProperties>,
}

impl common::Part for UpdateSlidePropertiesRequest {}

/// Updates the position of slides in the presentation.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateSlidesPositionRequest {
    /// The index where the slides should be inserted, based on the slide arrangement before the move takes place. Must be between zero and the number of slides in the presentation, inclusive.
    #[serde(rename = "insertionIndex")]
    pub insertion_index: Option<i32>,
    /// The IDs of the slides in the presentation that should be moved. The slides in this list must be in existing presentation order, without duplicates.
    #[serde(rename = "slideObjectIds")]
    pub slide_object_ids: Option<Vec<String>>,
}

impl common::Part for UpdateSlidesPositionRequest {}

/// Updates the properties of the table borders in a Table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateTableBorderPropertiesRequest {
    /// The border position in the table range the updates should apply to. If a border position is not specified, the updates will apply to all borders in the table range.
    #[serde(rename = "borderPosition")]
    pub border_position: Option<String>,
    /// The fields that should be updated. At least one field must be specified. The root `tableBorderProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the table border solid fill color, set `fields` to `"tableBorderFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The table border properties to update.
    #[serde(rename = "tableBorderProperties")]
    pub table_border_properties: Option<TableBorderProperties>,
    /// The table range representing the subset of the table to which the updates are applied. If a table range is not specified, the updates will apply to the entire table.
    #[serde(rename = "tableRange")]
    pub table_range: Option<TableRange>,
}

impl common::Part for UpdateTableBorderPropertiesRequest {}

/// Update the properties of a TableCell.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateTableCellPropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `tableCellProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the table cell background solid fill color, set `fields` to `"tableCellBackgroundFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The table cell properties to update.
    #[serde(rename = "tableCellProperties")]
    pub table_cell_properties: Option<TableCellProperties>,
    /// The table range representing the subset of the table to which the updates are applied. If a table range is not specified, the updates will apply to the entire table.
    #[serde(rename = "tableRange")]
    pub table_range: Option<TableRange>,
}

impl common::Part for UpdateTableCellPropertiesRequest {}

/// Updates the properties of a Table column.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateTableColumnPropertiesRequest {
    /// The list of zero-based indices specifying which columns to update. If no indices are provided, all columns in the table will be updated.
    #[serde(rename = "columnIndices")]
    pub column_indices: Option<Vec<i32>>,
    /// The fields that should be updated. At least one field must be specified. The root `tableColumnProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the column width, set `fields` to `"column_width"`. If '"column_width"' is included in the field mask but the property is left unset, the column width will default to 406,400 EMU (32 points).
    pub fields: Option<common::FieldMask>,
    /// The object ID of the table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The table column properties to update. If the value of `table_column_properties#column_width` in the request is less than 406,400 EMU (32 points), a 400 bad request error is returned.
    #[serde(rename = "tableColumnProperties")]
    pub table_column_properties: Option<TableColumnProperties>,
}

impl common::Part for UpdateTableColumnPropertiesRequest {}

/// Updates the properties of a Table row.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateTableRowPropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `tableRowProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the minimum row height, set `fields` to `"min_row_height"`. If '"min_row_height"' is included in the field mask but the property is left unset, the minimum row height will default to 0.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the table.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The list of zero-based indices specifying which rows to update. If no indices are provided, all rows in the table will be updated.
    #[serde(rename = "rowIndices")]
    pub row_indices: Option<Vec<i32>>,
    /// The table row properties to update.
    #[serde(rename = "tableRowProperties")]
    pub table_row_properties: Option<TableRowProperties>,
}

impl common::Part for UpdateTableRowPropertiesRequest {}

/// Update the styling of text in a Shape or Table.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateTextStyleRequest {
    /// The location of the cell in the table containing the text to style. If `object_id` refers to a table, `cell_location` must have a value. Otherwise, it must not.
    #[serde(rename = "cellLocation")]
    pub cell_location: Option<TableCellLocation>,
    /// The fields that should be updated. At least one field must be specified. The root `style` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example, to update the text style to bold, set `fields` to `"bold"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the shape or table with the text to be styled.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The style(s) to set on the text. If the value for a particular style matches that of the parent, that style will be set to inherit. Certain text style changes may cause other changes meant to mirror the behavior of the Slides editor. See the documentation of TextStyle for more information.
    pub style: Option<TextStyle>,
    /// The range of text to style. The range may be extended to include adjacent newlines. If the range fully contains a paragraph belonging to a list, the paragraph's bullet is also updated with the matching text style.
    #[serde(rename = "textRange")]
    pub text_range: Option<Range>,
}

impl common::Part for UpdateTextStyleRequest {}

/// Update the properties of a Video.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateVideoPropertiesRequest {
    /// The fields that should be updated. At least one field must be specified. The root `videoProperties` is implied and should not be specified. A single `"*"` can be used as short-hand for listing every field. For example to update the video outline color, set `fields` to `"outline.outlineFill.solidFill.color"`. To reset a property to its default value, include its field name in the field mask but leave the field itself unset.
    pub fields: Option<common::FieldMask>,
    /// The object ID of the video the updates are applied to.
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    /// The video properties to update.
    #[serde(rename = "videoProperties")]
    pub video_properties: Option<VideoProperties>,
}

impl common::Part for UpdateVideoPropertiesRequest {}

/// A PageElement kind representing a video.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Video {
    /// The video source's unique identifier for this video.
    pub id: Option<String>,
    /// The video source.
    pub source: Option<String>,
    /// An URL to a video. The URL is valid as long as the source video exists and sharing settings do not change.
    pub url: Option<String>,
    /// The properties of the video.
    #[serde(rename = "videoProperties")]
    pub video_properties: Option<VideoProperties>,
}

impl common::Part for Video {}

/// The properties of the Video.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VideoProperties {
    /// Whether to enable video autoplay when the page is displayed in present mode. Defaults to false.
    #[serde(rename = "autoPlay")]
    pub auto_play: Option<bool>,
    /// The time at which to end playback, measured in seconds from the beginning of the video. If set, the end time should be after the start time. If not set or if you set this to a value that exceeds the video's length, the video will be played until its end.
    pub end: Option<u32>,
    /// Whether to mute the audio during video playback. Defaults to false.
    pub mute: Option<bool>,
    /// The outline of the video. The default outline matches the defaults for new videos created in the Slides editor.
    pub outline: Option<Outline>,
    /// The time at which to start playback, measured in seconds from the beginning of the video. If set, the start time should be before the end time. If you set this to a value that exceeds the video's length in seconds, the video will be played from the last second. If not set, the video will be played from the beginning.
    pub start: Option<u32>,
}

impl common::Part for VideoProperties {}

/// Represents a font family and weight used to style a TextRun.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WeightedFontFamily {
    /// The font family of the text. The font family can be any font from the Font menu in Slides or from [Google Fonts] (https://fonts.google.com/). If the font name is unrecognized, the text is rendered in `Arial`.
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,
    /// The rendered weight of the text. This field can have any value that is a multiple of `100` between `100` and `900`, inclusive. This range corresponds to the numerical values described in the CSS 2.1 Specification, [section 15.6](https://www.w3.org/TR/CSS21/fonts.html#font-boldness), with non-numerical values disallowed. Weights greater than or equal to `700` are considered bold, and weights less than `700`are not bold. The default value is `400` ("normal").
    pub weight: Option<i32>,
}

impl common::Part for WeightedFontFamily {}

/// A PageElement kind representing word art.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WordArt {
    /// The text rendered as word art.
    #[serde(rename = "renderedText")]
    pub rendered_text: Option<String>,
}

impl common::Part for WordArt {}

/// Provides control over how write requests are executed.
///
/// This type is not used in any activity, and only used as *part* of another schema.
///
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde_with::serde_as]
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WriteControl {
    /// The revision ID of the presentation required for the write request. If specified and the required revision ID doesn't match the presentation's current revision ID, the request is not processed and returns a 400 bad request error. When a required revision ID is returned in a response, it indicates the revision ID of the document after the request was applied.
    #[serde(rename = "requiredRevisionId")]
    pub required_revision_id: Option<String>,
}

impl common::Part for WriteControl {}

// ###################
// MethodBuilders ###
// #################

/// A builder providing access to all methods supported on *presentation* resources.
/// It is not used directly, but through the [`Slides`] hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate google_slides1 as slides1;
///
/// # async fn dox() {
/// use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// let secret: yup_oauth2::ApplicationSecret = Default::default();
/// let connector = hyper_rustls::HttpsConnectorBuilder::new()
///     .with_native_roots()
///     .unwrap()
///     .https_only()
///     .enable_http2()
///     .build();
///
/// let executor = hyper_util::rt::TokioExecutor::new();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
///     secret,
///     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     yup_oauth2::client::CustomHyperClientBuilder::from(
///         hyper_util::client::legacy::Client::builder(executor).build(connector),
///     ),
/// ).build().await.unwrap();
///
/// let client = hyper_util::client::legacy::Client::builder(
///     hyper_util::rt::TokioExecutor::new()
/// )
/// .build(
///     hyper_rustls::HttpsConnectorBuilder::new()
///         .with_native_roots()
///         .unwrap()
///         .https_or_http()
///         .enable_http2()
///         .build()
/// );
/// let mut hub = Slides::new(client, auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `batch_update(...)`, `create(...)`, `get(...)`, `pages_get(...)` and `pages_get_thumbnail(...)`
/// // to build up your call.
/// let rb = hub.presentations();
/// # }
/// ```
pub struct PresentationMethods<'a, C>
where
    C: 'a,
{
    hub: &'a Slides<C>,
}

impl<'a, C> common::MethodsBuilder for PresentationMethods<'a, C> {}

impl<'a, C> PresentationMethods<'a, C> {
    /// Create a builder to help you perform the following task:
    ///
    /// Gets the latest version of the specified page in the presentation.
    ///
    /// # Arguments
    ///
    /// * `presentationId` - The ID of the presentation to retrieve.
    /// * `pageObjectId` - The object ID of the page to retrieve.
    pub fn pages_get(
        &self,
        presentation_id: &str,
        page_object_id: &str,
    ) -> PresentationPageGetCall<'a, C> {
        PresentationPageGetCall {
            hub: self.hub,
            _presentation_id: presentation_id.to_string(),
            _page_object_id: page_object_id.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }

    /// Create a builder to help you perform the following task:
    ///
    /// Generates a thumbnail of the latest version of the specified page in the presentation and returns a URL to the thumbnail image. This request counts as an [expensive read request](https://developers.google.com/workspace/slides/limits) for quota purposes.
    ///
    /// # Arguments
    ///
    /// * `presentationId` - The ID of the presentation to retrieve.
    /// * `pageObjectId` - The object ID of the page whose thumbnail to retrieve.
    pub fn pages_get_thumbnail(
        &self,
        presentation_id: &str,
        page_object_id: &str,
    ) -> PresentationPageGetThumbnailCall<'a, C> {
        PresentationPageGetThumbnailCall {
            hub: self.hub,
            _presentation_id: presentation_id.to_string(),
            _page_object_id: page_object_id.to_string(),
            _thumbnail_properties_thumbnail_size: Default::default(),
            _thumbnail_properties_mime_type: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }

    /// Create a builder to help you perform the following task:
    ///
    /// Applies one or more updates to the presentation. Each request is validated before being applied. If any request is not valid, then the entire request will fail and nothing will be applied. Some requests have replies to give you some information about how they are applied. Other requests do not need to return information; these each return an empty reply. The order of replies matches that of the requests. For example, suppose you call batchUpdate with four updates, and only the third one returns information. The response would have two empty replies: the reply to the third request, and another empty reply, in that order. Because other users may be editing the presentation, the presentation might not exactly reflect your changes: your changes may be altered with respect to collaborator changes. If there are no collaborators, the presentation should reflect your changes. In any case, the updates in your request are guaranteed to be applied together atomically.
    ///
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `presentationId` - The presentation to apply the updates to.
    pub fn batch_update(
        &self,
        request: BatchUpdatePresentationRequest,
        presentation_id: &str,
    ) -> PresentationBatchUpdateCall<'a, C> {
        PresentationBatchUpdateCall {
            hub: self.hub,
            _request: request,
            _presentation_id: presentation_id.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }

    /// Create a builder to help you perform the following task:
    ///
    /// Creates a blank presentation using the title given in the request. If a `presentationId` is provided, it is used as the ID of the new presentation. Otherwise, a new ID is generated. Other fields in the request, including any provided content, are ignored. Returns the created presentation.
    ///
    /// # Arguments
    ///
    /// * `request` - No description provided.
    pub fn create(&self, request: Presentation) -> PresentationCreateCall<'a, C> {
        PresentationCreateCall {
            hub: self.hub,
            _request: request,
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }

    /// Create a builder to help you perform the following task:
    ///
    /// Gets the latest version of the specified presentation.
    ///
    /// # Arguments
    ///
    /// * `presentationId` - The ID of the presentation to retrieve.
    pub fn get(&self, presentation_id: &str) -> PresentationGetCall<'a, C> {
        PresentationGetCall {
            hub: self.hub,
            _presentation_id: presentation_id.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}

// ###################
// CallBuilders   ###
// #################

/// Gets the latest version of the specified page in the presentation.
///
/// A builder for the *pages.get* method supported by a *presentation* resource.
/// It is not used directly, but through a [`PresentationMethods`] instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_slides1 as slides1;
/// # async fn dox() {
/// # use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// # let secret: yup_oauth2::ApplicationSecret = Default::default();
/// # let connector = hyper_rustls::HttpsConnectorBuilder::new()
/// #     .with_native_roots()
/// #     .unwrap()
/// #     .https_only()
/// #     .enable_http2()
/// #     .build();
///
/// # let executor = hyper_util::rt::TokioExecutor::new();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
/// #     secret,
/// #     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     yup_oauth2::client::CustomHyperClientBuilder::from(
/// #         hyper_util::client::legacy::Client::builder(executor).build(connector),
/// #     ),
/// # ).build().await.unwrap();
///
/// # let client = hyper_util::client::legacy::Client::builder(
/// #     hyper_util::rt::TokioExecutor::new()
/// # )
/// # .build(
/// #     hyper_rustls::HttpsConnectorBuilder::new()
/// #         .with_native_roots()
/// #         .unwrap()
/// #         .https_or_http()
/// #         .enable_http2()
/// #         .build()
/// # );
/// # let mut hub = Slides::new(client, auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.presentations().pages_get("presentationId", "pageObjectId")
///              .doit().await;
/// # }
/// ```
pub struct PresentationPageGetCall<'a, C>
where
    C: 'a,
{
    hub: &'a Slides<C>,
    _presentation_id: String,
    _page_object_id: String,
    _delegate: Option<&'a mut dyn common::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeSet<String>,
}

impl<'a, C> common::CallBuilder for PresentationPageGetCall<'a, C> {}

impl<'a, C> PresentationPageGetCall<'a, C>
where
    C: common::Connector,
{
    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> common::Result<(common::Response, Page)> {
        use std::borrow::Cow;
        use std::io::{Read, Seek};

        use common::{url::Params, ToParts};
        use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION, USER_AGENT};

        let mut dd = common::DefaultDelegate;
        let mut dlg: &mut dyn common::Delegate = self._delegate.unwrap_or(&mut dd);
        dlg.begin(common::MethodInfo {
            id: "slides.presentations.pages.get",
            http_method: hyper::Method::GET,
        });

        for &field in ["alt", "presentationId", "pageObjectId"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(common::Error::FieldClash(field));
            }
        }

        let mut params = Params::with_capacity(4 + self._additional_params.len());
        params.push("presentationId", self._presentation_id);
        params.push("pageObjectId", self._page_object_id);

        params.extend(self._additional_params.iter());

        params.push("alt", "json");
        let mut url =
            self.hub._base_url.clone() + "v1/presentations/{presentationId}/pages/{pageObjectId}";
        if self._scopes.is_empty() {
            self._scopes
                .insert(Scope::DriveReadonly.as_ref().to_string());
        }

        #[allow(clippy::single_element_loop)]
        for &(find_this, param_name) in [
            ("{presentationId}", "presentationId"),
            ("{pageObjectId}", "pageObjectId"),
        ]
        .iter()
        {
            url = params.uri_replacement(url, param_name, find_this, false);
        }
        {
            let to_remove = ["pageObjectId", "presentationId"];
            params.remove_params(&to_remove);
        }

        let url = params.parse_with_url(&url);

        loop {
            let token = match self
                .hub
                .auth
                .get_token(&self._scopes.iter().map(String::as_str).collect::<Vec<_>>()[..])
                .await
            {
                Ok(token) => token,
                Err(e) => match dlg.token(e) {
                    Ok(token) => token,
                    Err(e) => {
                        dlg.finished(false);
                        return Err(common::Error::MissingToken(e));
                    }
                },
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder()
                    .method(hyper::Method::GET)
                    .uri(url.as_str())
                    .header(USER_AGENT, self.hub._user_agent.clone());

                if let Some(token) = token.as_ref() {
                    req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
                }

                let request = req_builder
                    .header(CONTENT_LENGTH, 0_u64)
                    .body(common::to_body::<String>(None));

                client.request(request.unwrap()).await
            };

            match req_result {
                Err(err) => {
                    if let common::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d).await;
                        continue;
                    }
                    dlg.finished(false);
                    return Err(common::Error::HttpError(err));
                }
                Ok(res) => {
                    let (mut parts, body) = res.into_parts();
                    let mut body = common::Body::new(body);
                    if !parts.status.is_success() {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let error = serde_json::from_str(&common::to_string(&bytes));
                        let response = common::to_response(parts, bytes.into());

                        if let common::Retry::After(d) =
                            dlg.http_failure(&response, error.as_ref().ok())
                        {
                            sleep(d).await;
                            continue;
                        }

                        dlg.finished(false);

                        return Err(match error {
                            Ok(value) => common::Error::BadRequest(value),
                            _ => common::Error::Failure(response),
                        });
                    }
                    let response = {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let encoded = common::to_string(&bytes);
                        match serde_json::from_str(&encoded) {
                            Ok(decoded) => (common::to_response(parts, bytes.into()), decoded),
                            Err(error) => {
                                dlg.response_json_decode_error(&encoded, &error);
                                return Err(common::Error::JsonDecodeError(
                                    encoded.to_string(),
                                    error,
                                ));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(response);
                }
            }
        }
    }

    /// The ID of the presentation to retrieve.
    ///
    /// Sets the *presentation id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn presentation_id(mut self, new_value: &str) -> PresentationPageGetCall<'a, C> {
        self._presentation_id = new_value.to_string();
        self
    }
    /// The object ID of the page to retrieve.
    ///
    /// Sets the *page object id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn page_object_id(mut self, new_value: &str) -> PresentationPageGetCall<'a, C> {
        self._page_object_id = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    ///
    /// ````text
    ///                   It should be used to handle progress information, and to implement a certain level of resilience.
    /// ````
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(
        mut self,
        new_value: &'a mut dyn common::Delegate,
    ) -> PresentationPageGetCall<'a, C> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PresentationPageGetCall<'a, C>
    where
        T: AsRef<str>,
    {
        self._additional_params
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead of the default [`Scope`] variant
    /// [`Scope::DriveReadonly`].
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<St>(mut self, scope: St) -> PresentationPageGetCall<'a, C>
    where
        St: AsRef<str>,
    {
        self._scopes.insert(String::from(scope.as_ref()));
        self
    }
    /// Identifies the authorization scope(s) for the method you are building.
    ///
    /// See [`Self::add_scope()`] for details.
    pub fn add_scopes<I, St>(mut self, scopes: I) -> PresentationPageGetCall<'a, C>
    where
        I: IntoIterator<Item = St>,
        St: AsRef<str>,
    {
        self._scopes
            .extend(scopes.into_iter().map(|s| String::from(s.as_ref())));
        self
    }

    /// Removes all scopes, and no default scope will be used either.
    /// In this case, you have to specify your API-key using the `key` parameter (see [`Self::param()`]
    /// for details).
    pub fn clear_scopes(mut self) -> PresentationPageGetCall<'a, C> {
        self._scopes.clear();
        self
    }
}

/// Generates a thumbnail of the latest version of the specified page in the presentation and returns a URL to the thumbnail image. This request counts as an [expensive read request](https://developers.google.com/workspace/slides/limits) for quota purposes.
///
/// A builder for the *pages.getThumbnail* method supported by a *presentation* resource.
/// It is not used directly, but through a [`PresentationMethods`] instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_slides1 as slides1;
/// # async fn dox() {
/// # use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// # let secret: yup_oauth2::ApplicationSecret = Default::default();
/// # let connector = hyper_rustls::HttpsConnectorBuilder::new()
/// #     .with_native_roots()
/// #     .unwrap()
/// #     .https_only()
/// #     .enable_http2()
/// #     .build();
///
/// # let executor = hyper_util::rt::TokioExecutor::new();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
/// #     secret,
/// #     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     yup_oauth2::client::CustomHyperClientBuilder::from(
/// #         hyper_util::client::legacy::Client::builder(executor).build(connector),
/// #     ),
/// # ).build().await.unwrap();
///
/// # let client = hyper_util::client::legacy::Client::builder(
/// #     hyper_util::rt::TokioExecutor::new()
/// # )
/// # .build(
/// #     hyper_rustls::HttpsConnectorBuilder::new()
/// #         .with_native_roots()
/// #         .unwrap()
/// #         .https_or_http()
/// #         .enable_http2()
/// #         .build()
/// # );
/// # let mut hub = Slides::new(client, auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.presentations().pages_get_thumbnail("presentationId", "pageObjectId")
///              .thumbnail_properties_thumbnail_size("eos")
///              .thumbnail_properties_mime_type("dolor")
///              .doit().await;
/// # }
/// ```
pub struct PresentationPageGetThumbnailCall<'a, C>
where
    C: 'a,
{
    hub: &'a Slides<C>,
    _presentation_id: String,
    _page_object_id: String,
    _thumbnail_properties_thumbnail_size: Option<String>,
    _thumbnail_properties_mime_type: Option<String>,
    _delegate: Option<&'a mut dyn common::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeSet<String>,
}

impl<'a, C> common::CallBuilder for PresentationPageGetThumbnailCall<'a, C> {}

impl<'a, C> PresentationPageGetThumbnailCall<'a, C>
where
    C: common::Connector,
{
    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> common::Result<(common::Response, Thumbnail)> {
        use std::borrow::Cow;
        use std::io::{Read, Seek};

        use common::{url::Params, ToParts};
        use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION, USER_AGENT};

        let mut dd = common::DefaultDelegate;
        let mut dlg: &mut dyn common::Delegate = self._delegate.unwrap_or(&mut dd);
        dlg.begin(common::MethodInfo {
            id: "slides.presentations.pages.getThumbnail",
            http_method: hyper::Method::GET,
        });

        for &field in [
            "alt",
            "presentationId",
            "pageObjectId",
            "thumbnailProperties.thumbnailSize",
            "thumbnailProperties.mimeType",
        ]
        .iter()
        {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(common::Error::FieldClash(field));
            }
        }

        let mut params = Params::with_capacity(6 + self._additional_params.len());
        params.push("presentationId", self._presentation_id);
        params.push("pageObjectId", self._page_object_id);
        if let Some(value) = self._thumbnail_properties_thumbnail_size.as_ref() {
            params.push("thumbnailProperties.thumbnailSize", value);
        }
        if let Some(value) = self._thumbnail_properties_mime_type.as_ref() {
            params.push("thumbnailProperties.mimeType", value);
        }

        params.extend(self._additional_params.iter());

        params.push("alt", "json");
        let mut url = self.hub._base_url.clone()
            + "v1/presentations/{presentationId}/pages/{pageObjectId}/thumbnail";
        if self._scopes.is_empty() {
            self._scopes
                .insert(Scope::DriveReadonly.as_ref().to_string());
        }

        #[allow(clippy::single_element_loop)]
        for &(find_this, param_name) in [
            ("{presentationId}", "presentationId"),
            ("{pageObjectId}", "pageObjectId"),
        ]
        .iter()
        {
            url = params.uri_replacement(url, param_name, find_this, false);
        }
        {
            let to_remove = ["pageObjectId", "presentationId"];
            params.remove_params(&to_remove);
        }

        let url = params.parse_with_url(&url);

        loop {
            let token = match self
                .hub
                .auth
                .get_token(&self._scopes.iter().map(String::as_str).collect::<Vec<_>>()[..])
                .await
            {
                Ok(token) => token,
                Err(e) => match dlg.token(e) {
                    Ok(token) => token,
                    Err(e) => {
                        dlg.finished(false);
                        return Err(common::Error::MissingToken(e));
                    }
                },
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder()
                    .method(hyper::Method::GET)
                    .uri(url.as_str())
                    .header(USER_AGENT, self.hub._user_agent.clone());

                if let Some(token) = token.as_ref() {
                    req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
                }

                let request = req_builder
                    .header(CONTENT_LENGTH, 0_u64)
                    .body(common::to_body::<String>(None));

                client.request(request.unwrap()).await
            };

            match req_result {
                Err(err) => {
                    if let common::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d).await;
                        continue;
                    }
                    dlg.finished(false);
                    return Err(common::Error::HttpError(err));
                }
                Ok(res) => {
                    let (mut parts, body) = res.into_parts();
                    let mut body = common::Body::new(body);
                    if !parts.status.is_success() {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let error = serde_json::from_str(&common::to_string(&bytes));
                        let response = common::to_response(parts, bytes.into());

                        if let common::Retry::After(d) =
                            dlg.http_failure(&response, error.as_ref().ok())
                        {
                            sleep(d).await;
                            continue;
                        }

                        dlg.finished(false);

                        return Err(match error {
                            Ok(value) => common::Error::BadRequest(value),
                            _ => common::Error::Failure(response),
                        });
                    }
                    let response = {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let encoded = common::to_string(&bytes);
                        match serde_json::from_str(&encoded) {
                            Ok(decoded) => (common::to_response(parts, bytes.into()), decoded),
                            Err(error) => {
                                dlg.response_json_decode_error(&encoded, &error);
                                return Err(common::Error::JsonDecodeError(
                                    encoded.to_string(),
                                    error,
                                ));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(response);
                }
            }
        }
    }

    /// The ID of the presentation to retrieve.
    ///
    /// Sets the *presentation id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn presentation_id(mut self, new_value: &str) -> PresentationPageGetThumbnailCall<'a, C> {
        self._presentation_id = new_value.to_string();
        self
    }
    /// The object ID of the page whose thumbnail to retrieve.
    ///
    /// Sets the *page object id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn page_object_id(mut self, new_value: &str) -> PresentationPageGetThumbnailCall<'a, C> {
        self._page_object_id = new_value.to_string();
        self
    }
    /// The optional thumbnail image size. If you don't specify the size, the server chooses a default size of the image.
    ///
    /// Sets the *thumbnail properties.thumbnail size* query property to the given value.
    pub fn thumbnail_properties_thumbnail_size(
        mut self,
        new_value: &str,
    ) -> PresentationPageGetThumbnailCall<'a, C> {
        self._thumbnail_properties_thumbnail_size = Some(new_value.to_string());
        self
    }
    /// The optional mime type of the thumbnail image. If you don't specify the mime type, the mime type defaults to PNG.
    ///
    /// Sets the *thumbnail properties.mime type* query property to the given value.
    pub fn thumbnail_properties_mime_type(
        mut self,
        new_value: &str,
    ) -> PresentationPageGetThumbnailCall<'a, C> {
        self._thumbnail_properties_mime_type = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    ///
    /// ````text
    ///                   It should be used to handle progress information, and to implement a certain level of resilience.
    /// ````
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(
        mut self,
        new_value: &'a mut dyn common::Delegate,
    ) -> PresentationPageGetThumbnailCall<'a, C> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PresentationPageGetThumbnailCall<'a, C>
    where
        T: AsRef<str>,
    {
        self._additional_params
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead of the default [`Scope`] variant
    /// [`Scope::DriveReadonly`].
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<St>(mut self, scope: St) -> PresentationPageGetThumbnailCall<'a, C>
    where
        St: AsRef<str>,
    {
        self._scopes.insert(String::from(scope.as_ref()));
        self
    }
    /// Identifies the authorization scope(s) for the method you are building.
    ///
    /// See [`Self::add_scope()`] for details.
    pub fn add_scopes<I, St>(mut self, scopes: I) -> PresentationPageGetThumbnailCall<'a, C>
    where
        I: IntoIterator<Item = St>,
        St: AsRef<str>,
    {
        self._scopes
            .extend(scopes.into_iter().map(|s| String::from(s.as_ref())));
        self
    }

    /// Removes all scopes, and no default scope will be used either.
    /// In this case, you have to specify your API-key using the `key` parameter (see [`Self::param()`]
    /// for details).
    pub fn clear_scopes(mut self) -> PresentationPageGetThumbnailCall<'a, C> {
        self._scopes.clear();
        self
    }
}

/// Applies one or more updates to the presentation. Each request is validated before being applied. If any request is not valid, then the entire request will fail and nothing will be applied. Some requests have replies to give you some information about how they are applied. Other requests do not need to return information; these each return an empty reply. The order of replies matches that of the requests. For example, suppose you call batchUpdate with four updates, and only the third one returns information. The response would have two empty replies: the reply to the third request, and another empty reply, in that order. Because other users may be editing the presentation, the presentation might not exactly reflect your changes: your changes may be altered with respect to collaborator changes. If there are no collaborators, the presentation should reflect your changes. In any case, the updates in your request are guaranteed to be applied together atomically.
///
/// A builder for the *batchUpdate* method supported by a *presentation* resource.
/// It is not used directly, but through a [`PresentationMethods`] instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_slides1 as slides1;
/// use slides1::api::BatchUpdatePresentationRequest;
/// # async fn dox() {
/// # use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// # let secret: yup_oauth2::ApplicationSecret = Default::default();
/// # let connector = hyper_rustls::HttpsConnectorBuilder::new()
/// #     .with_native_roots()
/// #     .unwrap()
/// #     .https_only()
/// #     .enable_http2()
/// #     .build();
///
/// # let executor = hyper_util::rt::TokioExecutor::new();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
/// #     secret,
/// #     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     yup_oauth2::client::CustomHyperClientBuilder::from(
/// #         hyper_util::client::legacy::Client::builder(executor).build(connector),
/// #     ),
/// # ).build().await.unwrap();
///
/// # let client = hyper_util::client::legacy::Client::builder(
/// #     hyper_util::rt::TokioExecutor::new()
/// # )
/// # .build(
/// #     hyper_rustls::HttpsConnectorBuilder::new()
/// #         .with_native_roots()
/// #         .unwrap()
/// #         .https_or_http()
/// #         .enable_http2()
/// #         .build()
/// # );
/// # let mut hub = Slides::new(client, auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = BatchUpdatePresentationRequest::default();
///
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.presentations().batch_update(req, "presentationId")
///              .doit().await;
/// # }
/// ```
pub struct PresentationBatchUpdateCall<'a, C>
where
    C: 'a,
{
    hub: &'a Slides<C>,
    _request: BatchUpdatePresentationRequest,
    _presentation_id: String,
    _delegate: Option<&'a mut dyn common::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeSet<String>,
}

impl<'a, C> common::CallBuilder for PresentationBatchUpdateCall<'a, C> {}

impl<'a, C> PresentationBatchUpdateCall<'a, C>
where
    C: common::Connector,
{
    /// Perform the operation you have build so far.
    pub async fn doit(
        mut self,
    ) -> common::Result<(common::Response, BatchUpdatePresentationResponse)> {
        use std::borrow::Cow;
        use std::io::{Read, Seek};

        use common::{url::Params, ToParts};
        use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION, USER_AGENT};

        let mut dd = common::DefaultDelegate;
        let mut dlg: &mut dyn common::Delegate = self._delegate.unwrap_or(&mut dd);
        dlg.begin(common::MethodInfo {
            id: "slides.presentations.batchUpdate",
            http_method: hyper::Method::POST,
        });

        for &field in ["alt", "presentationId"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(common::Error::FieldClash(field));
            }
        }

        let mut params = Params::with_capacity(4 + self._additional_params.len());
        params.push("presentationId", self._presentation_id);

        params.extend(self._additional_params.iter());

        params.push("alt", "json");
        let mut url = self.hub._base_url.clone() + "v1/presentations/{presentationId}:batchUpdate";
        if self._scopes.is_empty() {
            self._scopes
                .insert(Scope::DriveReadonly.as_ref().to_string());
        }

        #[allow(clippy::single_element_loop)]
        for &(find_this, param_name) in [("{presentationId}", "presentationId")].iter() {
            url = params.uri_replacement(url, param_name, find_this, false);
        }
        {
            let to_remove = ["presentationId"];
            params.remove_params(&to_remove);
        }

        let url = params.parse_with_url(&url);

        let mut json_mime_type = mime::APPLICATION_JSON;
        let mut request_value_reader = {
            let mut value = serde_json::value::to_value(&self._request).expect("serde to work");
            common::remove_json_null_values(&mut value);
            let mut dst = std::io::Cursor::new(Vec::with_capacity(128));
            serde_json::to_writer(&mut dst, &value).unwrap();
            dst
        };
        let request_size = request_value_reader
            .seek(std::io::SeekFrom::End(0))
            .unwrap();
        request_value_reader
            .seek(std::io::SeekFrom::Start(0))
            .unwrap();

        loop {
            let token = match self
                .hub
                .auth
                .get_token(&self._scopes.iter().map(String::as_str).collect::<Vec<_>>()[..])
                .await
            {
                Ok(token) => token,
                Err(e) => match dlg.token(e) {
                    Ok(token) => token,
                    Err(e) => {
                        dlg.finished(false);
                        return Err(common::Error::MissingToken(e));
                    }
                },
            };
            request_value_reader
                .seek(std::io::SeekFrom::Start(0))
                .unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder()
                    .method(hyper::Method::POST)
                    .uri(url.as_str())
                    .header(USER_AGENT, self.hub._user_agent.clone());

                if let Some(token) = token.as_ref() {
                    req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
                }

                let request = req_builder
                    .header(CONTENT_TYPE, json_mime_type.to_string())
                    .header(CONTENT_LENGTH, request_size as u64)
                    .body(common::to_body(
                        request_value_reader.get_ref().clone().into(),
                    ));

                client.request(request.unwrap()).await
            };

            match req_result {
                Err(err) => {
                    if let common::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d).await;
                        continue;
                    }
                    dlg.finished(false);
                    return Err(common::Error::HttpError(err));
                }
                Ok(res) => {
                    let (mut parts, body) = res.into_parts();
                    let mut body = common::Body::new(body);
                    if !parts.status.is_success() {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let error = serde_json::from_str(&common::to_string(&bytes));
                        let response = common::to_response(parts, bytes.into());

                        if let common::Retry::After(d) =
                            dlg.http_failure(&response, error.as_ref().ok())
                        {
                            sleep(d).await;
                            continue;
                        }

                        dlg.finished(false);

                        return Err(match error {
                            Ok(value) => common::Error::BadRequest(value),
                            _ => common::Error::Failure(response),
                        });
                    }
                    let response = {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let encoded = common::to_string(&bytes);
                        match serde_json::from_str(&encoded) {
                            Ok(decoded) => (common::to_response(parts, bytes.into()), decoded),
                            Err(error) => {
                                dlg.response_json_decode_error(&encoded, &error);
                                return Err(common::Error::JsonDecodeError(
                                    encoded.to_string(),
                                    error,
                                ));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(response);
                }
            }
        }
    }

    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(
        mut self,
        new_value: BatchUpdatePresentationRequest,
    ) -> PresentationBatchUpdateCall<'a, C> {
        self._request = new_value;
        self
    }
    /// The presentation to apply the updates to.
    ///
    /// Sets the *presentation id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn presentation_id(mut self, new_value: &str) -> PresentationBatchUpdateCall<'a, C> {
        self._presentation_id = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    ///
    /// ````text
    ///                   It should be used to handle progress information, and to implement a certain level of resilience.
    /// ````
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(
        mut self,
        new_value: &'a mut dyn common::Delegate,
    ) -> PresentationBatchUpdateCall<'a, C> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PresentationBatchUpdateCall<'a, C>
    where
        T: AsRef<str>,
    {
        self._additional_params
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead of the default [`Scope`] variant
    /// [`Scope::DriveReadonly`].
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<St>(mut self, scope: St) -> PresentationBatchUpdateCall<'a, C>
    where
        St: AsRef<str>,
    {
        self._scopes.insert(String::from(scope.as_ref()));
        self
    }
    /// Identifies the authorization scope(s) for the method you are building.
    ///
    /// See [`Self::add_scope()`] for details.
    pub fn add_scopes<I, St>(mut self, scopes: I) -> PresentationBatchUpdateCall<'a, C>
    where
        I: IntoIterator<Item = St>,
        St: AsRef<str>,
    {
        self._scopes
            .extend(scopes.into_iter().map(|s| String::from(s.as_ref())));
        self
    }

    /// Removes all scopes, and no default scope will be used either.
    /// In this case, you have to specify your API-key using the `key` parameter (see [`Self::param()`]
    /// for details).
    pub fn clear_scopes(mut self) -> PresentationBatchUpdateCall<'a, C> {
        self._scopes.clear();
        self
    }
}

/// Creates a blank presentation using the title given in the request. If a `presentationId` is provided, it is used as the ID of the new presentation. Otherwise, a new ID is generated. Other fields in the request, including any provided content, are ignored. Returns the created presentation.
///
/// A builder for the *create* method supported by a *presentation* resource.
/// It is not used directly, but through a [`PresentationMethods`] instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_slides1 as slides1;
/// use slides1::api::Presentation;
/// # async fn dox() {
/// # use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// # let secret: yup_oauth2::ApplicationSecret = Default::default();
/// # let connector = hyper_rustls::HttpsConnectorBuilder::new()
/// #     .with_native_roots()
/// #     .unwrap()
/// #     .https_only()
/// #     .enable_http2()
/// #     .build();
///
/// # let executor = hyper_util::rt::TokioExecutor::new();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
/// #     secret,
/// #     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     yup_oauth2::client::CustomHyperClientBuilder::from(
/// #         hyper_util::client::legacy::Client::builder(executor).build(connector),
/// #     ),
/// # ).build().await.unwrap();
///
/// # let client = hyper_util::client::legacy::Client::builder(
/// #     hyper_util::rt::TokioExecutor::new()
/// # )
/// # .build(
/// #     hyper_rustls::HttpsConnectorBuilder::new()
/// #         .with_native_roots()
/// #         .unwrap()
/// #         .https_or_http()
/// #         .enable_http2()
/// #         .build()
/// # );
/// # let mut hub = Slides::new(client, auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = Presentation::default();
///
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.presentations().create(req)
///              .doit().await;
/// # }
/// ```
pub struct PresentationCreateCall<'a, C>
where
    C: 'a,
{
    hub: &'a Slides<C>,
    _request: Presentation,
    _delegate: Option<&'a mut dyn common::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeSet<String>,
}

impl<'a, C> common::CallBuilder for PresentationCreateCall<'a, C> {}

impl<'a, C> PresentationCreateCall<'a, C>
where
    C: common::Connector,
{
    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> common::Result<(common::Response, Presentation)> {
        use std::borrow::Cow;
        use std::io::{Read, Seek};

        use common::{url::Params, ToParts};
        use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION, USER_AGENT};

        let mut dd = common::DefaultDelegate;
        let mut dlg: &mut dyn common::Delegate = self._delegate.unwrap_or(&mut dd);
        dlg.begin(common::MethodInfo {
            id: "slides.presentations.create",
            http_method: hyper::Method::POST,
        });

        for &field in ["alt"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(common::Error::FieldClash(field));
            }
        }

        let mut params = Params::with_capacity(3 + self._additional_params.len());

        params.extend(self._additional_params.iter());

        params.push("alt", "json");
        let mut url = self.hub._base_url.clone() + "v1/presentations";
        if self._scopes.is_empty() {
            self._scopes.insert(Scope::Drive.as_ref().to_string());
        }

        let url = params.parse_with_url(&url);

        let mut json_mime_type = mime::APPLICATION_JSON;
        let mut request_value_reader = {
            let mut value = serde_json::value::to_value(&self._request).expect("serde to work");
            common::remove_json_null_values(&mut value);
            let mut dst = std::io::Cursor::new(Vec::with_capacity(128));
            serde_json::to_writer(&mut dst, &value).unwrap();
            dst
        };
        let request_size = request_value_reader
            .seek(std::io::SeekFrom::End(0))
            .unwrap();
        request_value_reader
            .seek(std::io::SeekFrom::Start(0))
            .unwrap();

        loop {
            let token = match self
                .hub
                .auth
                .get_token(&self._scopes.iter().map(String::as_str).collect::<Vec<_>>()[..])
                .await
            {
                Ok(token) => token,
                Err(e) => match dlg.token(e) {
                    Ok(token) => token,
                    Err(e) => {
                        dlg.finished(false);
                        return Err(common::Error::MissingToken(e));
                    }
                },
            };
            request_value_reader
                .seek(std::io::SeekFrom::Start(0))
                .unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder()
                    .method(hyper::Method::POST)
                    .uri(url.as_str())
                    .header(USER_AGENT, self.hub._user_agent.clone());

                if let Some(token) = token.as_ref() {
                    req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
                }

                let request = req_builder
                    .header(CONTENT_TYPE, json_mime_type.to_string())
                    .header(CONTENT_LENGTH, request_size as u64)
                    .body(common::to_body(
                        request_value_reader.get_ref().clone().into(),
                    ));

                client.request(request.unwrap()).await
            };

            match req_result {
                Err(err) => {
                    if let common::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d).await;
                        continue;
                    }
                    dlg.finished(false);
                    return Err(common::Error::HttpError(err));
                }
                Ok(res) => {
                    let (mut parts, body) = res.into_parts();
                    let mut body = common::Body::new(body);
                    if !parts.status.is_success() {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let error = serde_json::from_str(&common::to_string(&bytes));
                        let response = common::to_response(parts, bytes.into());

                        if let common::Retry::After(d) =
                            dlg.http_failure(&response, error.as_ref().ok())
                        {
                            sleep(d).await;
                            continue;
                        }

                        dlg.finished(false);

                        return Err(match error {
                            Ok(value) => common::Error::BadRequest(value),
                            _ => common::Error::Failure(response),
                        });
                    }
                    let response = {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let encoded = common::to_string(&bytes);
                        match serde_json::from_str(&encoded) {
                            Ok(decoded) => (common::to_response(parts, bytes.into()), decoded),
                            Err(error) => {
                                dlg.response_json_decode_error(&encoded, &error);
                                return Err(common::Error::JsonDecodeError(
                                    encoded.to_string(),
                                    error,
                                ));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(response);
                }
            }
        }
    }

    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: Presentation) -> PresentationCreateCall<'a, C> {
        self._request = new_value;
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    ///
    /// ````text
    ///                   It should be used to handle progress information, and to implement a certain level of resilience.
    /// ````
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(
        mut self,
        new_value: &'a mut dyn common::Delegate,
    ) -> PresentationCreateCall<'a, C> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PresentationCreateCall<'a, C>
    where
        T: AsRef<str>,
    {
        self._additional_params
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead of the default [`Scope`] variant
    /// [`Scope::Drive`].
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<St>(mut self, scope: St) -> PresentationCreateCall<'a, C>
    where
        St: AsRef<str>,
    {
        self._scopes.insert(String::from(scope.as_ref()));
        self
    }
    /// Identifies the authorization scope(s) for the method you are building.
    ///
    /// See [`Self::add_scope()`] for details.
    pub fn add_scopes<I, St>(mut self, scopes: I) -> PresentationCreateCall<'a, C>
    where
        I: IntoIterator<Item = St>,
        St: AsRef<str>,
    {
        self._scopes
            .extend(scopes.into_iter().map(|s| String::from(s.as_ref())));
        self
    }

    /// Removes all scopes, and no default scope will be used either.
    /// In this case, you have to specify your API-key using the `key` parameter (see [`Self::param()`]
    /// for details).
    pub fn clear_scopes(mut self) -> PresentationCreateCall<'a, C> {
        self._scopes.clear();
        self
    }
}

/// Gets the latest version of the specified presentation.
///
/// A builder for the *get* method supported by a *presentation* resource.
/// It is not used directly, but through a [`PresentationMethods`] instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_slides1 as slides1;
/// # async fn dox() {
/// # use slides1::{Slides, FieldMask, hyper_rustls, hyper_util, yup_oauth2};
///
/// # let secret: yup_oauth2::ApplicationSecret = Default::default();
/// # let connector = hyper_rustls::HttpsConnectorBuilder::new()
/// #     .with_native_roots()
/// #     .unwrap()
/// #     .https_only()
/// #     .enable_http2()
/// #     .build();
///
/// # let executor = hyper_util::rt::TokioExecutor::new();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
/// #     secret,
/// #     yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     yup_oauth2::client::CustomHyperClientBuilder::from(
/// #         hyper_util::client::legacy::Client::builder(executor).build(connector),
/// #     ),
/// # ).build().await.unwrap();
///
/// # let client = hyper_util::client::legacy::Client::builder(
/// #     hyper_util::rt::TokioExecutor::new()
/// # )
/// # .build(
/// #     hyper_rustls::HttpsConnectorBuilder::new()
/// #         .with_native_roots()
/// #         .unwrap()
/// #         .https_or_http()
/// #         .enable_http2()
/// #         .build()
/// # );
/// # let mut hub = Slides::new(client, auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.presentations().get("presentationId")
///              .doit().await;
/// # }
/// ```
pub struct PresentationGetCall<'a, C>
where
    C: 'a,
{
    hub: &'a Slides<C>,
    _presentation_id: String,
    _delegate: Option<&'a mut dyn common::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeSet<String>,
}

impl<'a, C> common::CallBuilder for PresentationGetCall<'a, C> {}

impl<'a, C> PresentationGetCall<'a, C>
where
    C: common::Connector,
{
    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> common::Result<(common::Response, Presentation)> {
        use std::borrow::Cow;
        use std::io::{Read, Seek};

        use common::{url::Params, ToParts};
        use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION, USER_AGENT};

        let mut dd = common::DefaultDelegate;
        let mut dlg: &mut dyn common::Delegate = self._delegate.unwrap_or(&mut dd);
        dlg.begin(common::MethodInfo {
            id: "slides.presentations.get",
            http_method: hyper::Method::GET,
        });

        for &field in ["alt", "presentationId"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(common::Error::FieldClash(field));
            }
        }

        let mut params = Params::with_capacity(3 + self._additional_params.len());
        params.push("presentationId", self._presentation_id);

        params.extend(self._additional_params.iter());

        params.push("alt", "json");
        let mut url = self.hub._base_url.clone() + "v1/presentations/{+presentationId}";
        if self._scopes.is_empty() {
            self._scopes
                .insert(Scope::DriveReadonly.as_ref().to_string());
        }

        #[allow(clippy::single_element_loop)]
        for &(find_this, param_name) in [("{+presentationId}", "presentationId")].iter() {
            url = params.uri_replacement(url, param_name, find_this, true);
        }
        {
            let to_remove = ["presentationId"];
            params.remove_params(&to_remove);
        }

        let url = params.parse_with_url(&url);

        loop {
            let token = match self
                .hub
                .auth
                .get_token(&self._scopes.iter().map(String::as_str).collect::<Vec<_>>()[..])
                .await
            {
                Ok(token) => token,
                Err(e) => match dlg.token(e) {
                    Ok(token) => token,
                    Err(e) => {
                        dlg.finished(false);
                        return Err(common::Error::MissingToken(e));
                    }
                },
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder()
                    .method(hyper::Method::GET)
                    .uri(url.as_str())
                    .header(USER_AGENT, self.hub._user_agent.clone());

                if let Some(token) = token.as_ref() {
                    req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
                }

                let request = req_builder
                    .header(CONTENT_LENGTH, 0_u64)
                    .body(common::to_body::<String>(None));

                client.request(request.unwrap()).await
            };

            match req_result {
                Err(err) => {
                    if let common::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d).await;
                        continue;
                    }
                    dlg.finished(false);
                    return Err(common::Error::HttpError(err));
                }
                Ok(res) => {
                    let (mut parts, body) = res.into_parts();
                    let mut body = common::Body::new(body);
                    if !parts.status.is_success() {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let error = serde_json::from_str(&common::to_string(&bytes));
                        let response = common::to_response(parts, bytes.into());

                        if let common::Retry::After(d) =
                            dlg.http_failure(&response, error.as_ref().ok())
                        {
                            sleep(d).await;
                            continue;
                        }

                        dlg.finished(false);

                        return Err(match error {
                            Ok(value) => common::Error::BadRequest(value),
                            _ => common::Error::Failure(response),
                        });
                    }
                    let response = {
                        let bytes = common::to_bytes(body).await.unwrap_or_default();
                        let encoded = common::to_string(&bytes);
                        match serde_json::from_str(&encoded) {
                            Ok(decoded) => (common::to_response(parts, bytes.into()), decoded),
                            Err(error) => {
                                dlg.response_json_decode_error(&encoded, &error);
                                return Err(common::Error::JsonDecodeError(
                                    encoded.to_string(),
                                    error,
                                ));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(response);
                }
            }
        }
    }

    /// The ID of the presentation to retrieve.
    ///
    /// Sets the *presentation id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn presentation_id(mut self, new_value: &str) -> PresentationGetCall<'a, C> {
        self._presentation_id = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    ///
    /// ````text
    ///                   It should be used to handle progress information, and to implement a certain level of resilience.
    /// ````
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(
        mut self,
        new_value: &'a mut dyn common::Delegate,
    ) -> PresentationGetCall<'a, C> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PresentationGetCall<'a, C>
    where
        T: AsRef<str>,
    {
        self._additional_params
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead of the default [`Scope`] variant
    /// [`Scope::DriveReadonly`].
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<St>(mut self, scope: St) -> PresentationGetCall<'a, C>
    where
        St: AsRef<str>,
    {
        self._scopes.insert(String::from(scope.as_ref()));
        self
    }
    /// Identifies the authorization scope(s) for the method you are building.
    ///
    /// See [`Self::add_scope()`] for details.
    pub fn add_scopes<I, St>(mut self, scopes: I) -> PresentationGetCall<'a, C>
    where
        I: IntoIterator<Item = St>,
        St: AsRef<str>,
    {
        self._scopes
            .extend(scopes.into_iter().map(|s| String::from(s.as_ref())));
        self
    }

    /// Removes all scopes, and no default scope will be used either.
    /// In this case, you have to specify your API-key using the `key` parameter (see [`Self::param()`]
    /// for details).
    pub fn clear_scopes(mut self) -> PresentationGetCall<'a, C> {
        self._scopes.clear();
        self
    }
}
