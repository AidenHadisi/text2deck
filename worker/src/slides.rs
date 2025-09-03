use crate::{oauth::Token, splitter::Splitter};
use serde::{Deserialize, Serialize};
use validator::Validate;
use worker::{Fetch, Headers, Method, Request as WorkerRequest, RequestInit, Result};

const API_BASE: &str = "https://slides.googleapis.com/v1";

/// Represents a request to create slides from text content.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSlidesRequest {
    #[validate(length(min = 1, max = 100))]
    pub title: String,

    #[validate(length(min = 1))]
    pub content: String,

    pub splitter: Splitter,
}

/// Google Slides API structures
#[derive(Debug, Serialize, Deserialize)]
struct CreatePresentationRequest {
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Presentation {
    presentation_id: String,
    title: String,
    slides: Vec<Slide>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Slide {
    object_id: String,
    slide_properties: SlideProperties,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlideProperties {
    layout_object_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchUpdateRequest {
    requests: Vec<UpdateRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    create_slide: Option<CreateSlideRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insert_text: Option<InsertTextRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSlideRequest {
    object_id: Option<String>,
    insertion_index: Option<i32>,
    slide_layout_reference: Option<SlideLayoutReference>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlideLayoutReference {
    layout_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertTextRequest {
    object_id: String,
    insertion_index: i32,
    text: String,
    cell_location: Option<TableCellLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableCellLocation {
    row_index: i32,
    column_index: i32,
}

/// Creates a new Google Slides presentation and populates it with content chunks.
pub async fn create_slides_from_text(
    token: &Token,
    request: &CreateSlidesRequest,
) -> Result<String> {
    // Validate request
    request
        .validate()
        .map_err(|e| worker::Error::from(e.to_string()))?;

    // Split the content into chunks
    let chunks = request.splitter.split(&request.content);

    if chunks.is_empty() {
        return Err(worker::Error::from("No content chunks generated"));
    }

    if chunks.len() > 100 {
        return Err(worker::Error::from("Too many slides (max 100)"));
    }

    // Create the presentation
    let presentation_id = create_presentation(token, &request.title).await?;

    // Add slides for each chunk (skip the first slide as it's created by default)
    populate_slides(token, &presentation_id, &chunks).await?;

    Ok(presentation_id)
}

/// Creates a new Google Slides presentation with the given title.
async fn create_presentation(token: &Token, title: &str) -> Result<String> {
    let url = format!("{}/presentations", API_BASE);

    let create_request = CreatePresentationRequest {
        title: title.to_string(),
    };

    let body = serde_wasm_bindgen::to_value(&create_request)
        .map_err(|e| worker::Error::from(e.to_string()))?;

    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Authorization", &format!("Bearer {}", token.access_token))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_body(Some(body.into()))
        .with_headers(headers);

    let request = WorkerRequest::new_with_init(&url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() < 200 || response.status_code() >= 300 {
        let error_text = response.text().await?;
        return Err(worker::Error::from(format!(
            "Failed to create presentation: {}",
            error_text
        )));
    }

    let presentation: Presentation = response.json().await?;
    Ok(presentation.presentation_id)
}

/// Populates the presentation with slides containing the provided text chunks.
async fn populate_slides(token: &Token, presentation_id: &str, chunks: &[String]) -> Result<()> {
    let url = format!("{}/presentations/{}:batchUpdate", API_BASE, presentation_id);

    let mut requests = Vec::new();

    // For each chunk, create a new slide (except the first one, use the default slide)
    for (index, chunk) in chunks.iter().enumerate() {
        if index > 0 {
            // Create a new slide for chunks after the first one
            requests.push(UpdateRequest {
                create_slide: Some(CreateSlideRequest {
                    object_id: Some(format!("slide_{}", index)),
                    insertion_index: Some(index as i32 + 1),
                    slide_layout_reference: Some(SlideLayoutReference {
                        layout_id: "TITLE_AND_BODY".to_string(),
                    }),
                }),
                insert_text: None,
            });
        }

        // Add text to the slide
        // Note: In a real implementation, you would need to get the actual text box object IDs
        // This is a simplified version that assumes standard layout object IDs
        let text_box_id = if index == 0 {
            "g_placeholder_1".to_string() // Default slide title placeholder
        } else {
            format!("g_placeholder_{}_1", index + 1) // Title placeholder for new slides
        };

        requests.push(UpdateRequest {
            create_slide: None,
            insert_text: Some(InsertTextRequest {
                object_id: text_box_id,
                insertion_index: 0,
                text: chunk.clone(),
                cell_location: None,
            }),
        });
    }

    let batch_request = BatchUpdateRequest { requests };

    let body =
        serde_json::to_string(&batch_request).map_err(|e| worker::Error::from(e.to_string()))?;

    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Authorization", &format!("Bearer {}", token.access_token))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_body(Some(body.into()))
        .with_headers(headers);

    let request = WorkerRequest::new_with_init(&url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() < 200 || response.status_code() >= 300 {
        let error_text = response.text().await?;
        return Err(worker::Error::from(format!(
            "Failed to update slides: {}",
            error_text
        )));
    }

    Ok(())
}
