use crate::config::ImageConfig;
use crate::error_log;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use super::{ImageGenerationOptions, ImageJobResult, ImageJobStatus};

#[tauri::command]
pub async fn generate_all_images(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    options: ImageGenerationOptions,
    config: ImageConfig,
) -> Result<Vec<ImageJobResult>, String> {
    let cwd_path = cwd.inner().clone();
    let project_dir = cwd.join("projects").join(&project_name);
    
    if !project_dir.exists() {
        let error = format!("Project not found: {}", project_name);
        error_log::log_error(&cwd_path, &error);
        return Err(error);
    }
    
    error_log::log_info(&cwd_path, &format!("Starting image generation for project: {}", project_name));
    
    // Get style content if specified
    let style_content = if let Some(style_name) = &options.style {
        let style_path = cwd.join("styles").join(format!("{}.md", style_name));
        if style_path.exists() {
            Some(tokio::fs::read_to_string(&style_path).await.unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };
    
    // Find all page files
    let mut page_files = Vec::new();
    let mut entries = tokio::fs::read_dir(&project_dir)
        .await
        .map_err(|e| {
            let error = format!("Failed to read project directory: {}", e);
            error_log::log_error(&cwd_path, &error);
            error
        })?;
    
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("page-") && name.ends_with(".md") {
            page_files.push(entry.path());
        }
    }
    
    page_files.sort_by_key(|p| p.file_name().unwrap().to_string_lossy().to_string());
    
    let total_pages = page_files.len();
    
    let mut results = Vec::new();
    // Use width/height from options, fallback to 1920x1080 (16:9)
    let width = options.width.unwrap_or(1920);
    let height = options.height.unwrap_or(1080);
    
    for page_path in page_files {
        let page_name = page_path.file_stem().unwrap().to_string_lossy().to_string();
        let page_num: u32 = page_name
            .strip_prefix("page-")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        // Read page content
        let page_content = match tokio::fs::read_to_string(&page_path).await {
            Ok(content) => content,
            Err(e) => {
                let error = format!("Failed to read page file {}: {}", page_name, e);
                error_log::log_error(&cwd_path, &error);
                results.push(ImageJobResult {
                    page_num,
                    status: ImageJobStatus::Failed(error.clone()),
                    output_path: None,
                    error: Some(error),
                });
                continue;
            }
        };
        
        // Determine page type and get template image
        let page_type = determine_page_type(&page_content, page_num, total_pages);
        let template_image = get_template_image(&cwd, &options.template, page_type).await;
        
        // Build prompt
        let prompt = build_image_prompt(&page_content, style_content.as_deref(), page_type);
        
        // Generate image
        let output_path = project_dir.join(format!("page-{}.png", page_num));
        
        let result = if let Some(template_path) = template_image {
            // Use template as reference (image-to-image)
            generate_image_with_template(&prompt, width, height, &config, &output_path, &template_path).await
        } else {
            // Generate without template (text-to-image)
            generate_single_image(&prompt, width, height, &config, &output_path).await
        };
        
        let (status, output_path_str, error) = match result {
            Ok(_) => {
                error_log::log_info(&cwd_path, &format!("Image generated successfully for page {}", page_num));
                (ImageJobStatus::Success, Some(output_path.to_string_lossy().to_string()), None)
            },
            Err(e) => {
                error_log::log_error(&cwd_path, &format!("Failed to generate image for page {}: {}", page_num, e));
                (ImageJobStatus::Failed(e.clone()), None, Some(e))
            }
        };
        
        results.push(ImageJobResult {
            page_num,
            status,
            output_path: output_path_str,
            error,
        });
    }
    
    error_log::log_info(&cwd_path, &format!("Image generation completed for project: {}", project_name));
    Ok(results)
}

#[tauri::command]
pub async fn regenerate_image(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    page_num: u32,
    custom_prompt: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    config: ImageConfig,
) -> Result<String, String> {
    let project_dir = cwd.join("projects").join(&project_name);
    // Use two-digit format for page numbers
    let page_path = project_dir.join(format!("page-{:02}.md", page_num));
    
    if !page_path.exists() {
        return Err(format!("Page {} not found", page_num));
    }
    
    let page_content = tokio::fs::read_to_string(&page_path)
        .await
        .map_err(|e| format!("Failed to read page file: {}", e))?;
    
    let prompt = custom_prompt.unwrap_or_else(|| {
        let page_type = determine_page_type(&page_content, page_num, 0);
        build_image_prompt(&page_content, None, page_type)
    });
    
    // Use two-digit format for output path
    let output_path = project_dir.join(format!("page-{:02}.png", page_num));
    
    let img_width = width.unwrap_or(1920);
    let img_height = height.unwrap_or(1080);
    
    generate_single_image(&prompt, img_width, img_height, &config, &output_path).await?;
    
    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn generate_image(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    page_num: u32,
    template: Option<String>,
    style: Option<String>,
    width: u32,
    height: u32,
    config: ImageConfig,
) -> Result<String, String> {
    let cwd_path = cwd.inner().clone();
    let project_dir = cwd.join("projects").join(&project_name);
    // Use two-digit format for page numbers
    let page_path = project_dir.join(format!("page-{:02}.md", page_num));
    
    if !page_path.exists() {
        return Err(format!("Page {} not found", page_num));
    }
    
    // Get style content if specified
    let style_content = if let Some(style_name) = style {
        let style_path = cwd.join("styles").join(format!("{}.md", style_name));
        if style_path.exists() {
            Some(tokio::fs::read_to_string(&style_path).await.unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };
    
    // Read page content
    let page_content = tokio::fs::read_to_string(&page_path)
        .await
        .map_err(|e| format!("Failed to read page file: {}", e))?;
    
    // Determine page type and get template image
    let page_type = determine_page_type(&page_content, page_num, 0);
    let template_image = if let Some(template_name) = template {
        get_template_image(&cwd, &Some(template_name), page_type).await
    } else {
        None
    };
    
    // Build prompt
    let prompt = build_image_prompt(&page_content, style_content.as_deref(), page_type);
    
    // Generate image - use two-digit format for output path
    let output_path = project_dir.join(format!("page-{:02}.png", page_num));
    
    let result = if let Some(template_path) = template_image {
        generate_image_with_template(&prompt, width, height, &config, &output_path, &template_path).await
    } else {
        generate_single_image(&prompt, width, height, &config, &output_path).await
    };
    
    match result {
        Ok(_) => {
            error_log::log_info(&cwd_path, &format!("Image generated successfully for page {}", page_num));
            Ok(output_path.to_string_lossy().to_string())
        }
        Err(e) => {
            error_log::log_error(&cwd_path, &format!("Failed to generate image for page {}: {}", page_num, e));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn refine_image(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    page_num: u32,
    refine_prompt: String,
    width: u32,
    height: u32,
    config: ImageConfig,
) -> Result<String, String> {
    let project_dir = cwd.join("projects").join(&project_name);
    // Use two-digit format for output path
    let output_path = project_dir.join(format!("page-{:02}.png", page_num));
    
    if !output_path.exists() {
        return Err(format!("Image for page {} not found", page_num));
    }
    
    // Read existing image and encode as base64
    let image_data = tokio::fs::read(&output_path)
        .await
        .map_err(|e| format!("Failed to read image: {}", e))?;
    
    let base64_image = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
    
    // Call image editing API
    let client = reqwest::Client::new();
    
    let response = client
        .post(config.endpoint.replace("/generations", "/edits"))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&serde_json::json!({
            "image": base64_image,
            "prompt": refine_prompt,
            "n": 1,
            "size": format!("{}x{}", width, height),
        }))
        .send()
        .await
        .map_err(|e| format!("Image edit API request failed: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Image edit API error: {}", error_text));
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;
    
    // Try URL first, then base64
      let image_url = if let Some(url) = json["data"][0]["url"].as_str() {
          url.to_string()
      } else if let Some(b64) = json["data"][0]["b64_json"].as_str() {
          // Handle base64 response - decode and save directly
          let image_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
              .map_err(|e| format!("Failed to decode base64 image: {}", e))?;
          tokio::fs::write(&output_path, &image_bytes)
              .await
              .map_err(|e| format!("Failed to save image: {}", e))?;
          return Ok(output_path.to_string_lossy().to_string());
      } else {
          return Err(format!("Failed to extract image from response. Response keys: {:?}", 
              json.as_object().map(|m| m.keys().collect::<Vec<_>>())));
      };

      // Download and save the image
    let image_response = reqwest::get(image_url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;
    
    let image_bytes = image_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image data: {}", e))?;
    
    tokio::fs::write(&output_path, &image_bytes)
        .await
        .map_err(|e| format!("Failed to save image: {}", e))?;
    
    Ok(output_path.to_string_lossy().to_string())
}

/// Determine page type based on content and position
fn determine_page_type(content: &str, page_num: u32, total_pages: usize) -> &'static str {
    // First page is always front cover
    if page_num == 1 {
        return "front-cover";
    }
    
    // Last page is back cover (if we know total)
    if total_pages > 0 && page_num as usize == total_pages {
        return "back-cover";
    }
    
    // Check content for keywords
    let lower = content.to_lowercase();
    if lower.contains("封面") || lower.contains("cover") {
        return "front-cover";
    }
    if lower.contains("封底") || lower.contains("thank") || lower.contains("谢谢") || lower.contains("感谢") {
        return "back-cover";
    }
    
    "content"
}

/// Get template image path for the given page type
async fn get_template_image(cwd: &PathBuf, template_name: &Option<String>, page_type: &str) -> Option<PathBuf> {
    let template_name = template_name.as_ref()?;
    
    let template_dir = cwd.join("templates").join(template_name);
    if !template_dir.exists() {
        error_log::log_error(cwd, &format!("Template directory not found: {:?}", template_dir));
        return None;
    }
    
    let template_file = match page_type {
        "front-cover" => template_dir.join("front-cover.png"),
        "back-cover" => template_dir.join("back-cover.png"),
        _ => template_dir.join("content.png"),
    };
    
    if template_file.exists() {
        error_log::log_info(cwd, &format!("Using template image: {:?}", template_file));
        Some(template_file)
    } else {
        error_log::log_error(cwd, &format!("Template image not found: {:?}", template_file));
        None
    }
}

/// Build image generation prompt with Chinese descriptions
fn build_image_prompt(page_content: &str, style_content: Option<&str>, page_type: &str) -> String {
    let mut prompt_parts = Vec::new();
    
    // Add style if provided - use full content directly
    if let Some(style) = style_content {
        prompt_parts.push(format!("风格要求：\n{}", style));
    }
    
    // Add page type context in Chinese
    let page_type_desc = match page_type {
        "front-cover" => "这是一个封面页参考模板。请保持模板的固有元素及其大小不变（如logo、背景色块、导航布局、公司名称、保密提示等），只更新动态内容区域。",
        "back-cover" => "这是一个封底页参考模板。请保持模板的固有元素及其大小不变（如logo、背景色块、导航布局、公司名称、保密提示等），只更新动态内容区域。",
        _ => "这是一个内容页参考模板。请保持模板的固有元素及其大小不变（如logo、背景色块、导航布局、公司名称、保密提示等），只更新动态内容区域，确保信息清晰易读。",
    };
    prompt_parts.push(page_type_desc.to_string());
    
    // Use page content directly (raw markdown)
    prompt_parts.push(format!("页面内容：\n{}", page_content));
    
    // Add image generation guidance
    prompt_parts.push("\n请生成一张高质量的信息图表图片，尺寸比例适当，适合演示文稿使用。".to_string());
    
    prompt_parts.join("\n")
}

/// Generate image without template (text-to-image)
async fn generate_single_image(
    prompt: &str,
    width: u32,
    height: u32,
    config: &ImageConfig,
    output_path: &PathBuf,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Build request body - handle different API formats
    let request_body = if config.provider == "dashscope" {
        // DashScope (阿里云) format
        serde_json::json!({
            "model": config.model,
            "input": {
                "prompt": prompt,
            },
            "parameters": {
                "size": format!("{}x{}", width, height),
                "n": 1,
            }
        })
    } else {
        // OpenAI-compatible format (no response_format needed)
        serde_json::json!({
            "model": config.model,
            "prompt": prompt,
            "n": 1,
            "size": format!("{}x{}", width, height),
        })
    };
    
    let mut request = client
        .post(&config.endpoint)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json");
    
    // Add extra headers
    for header in &config.extra_headers {
        request = request.header(&header.key, &header.value);
    }
    
    let response = request
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Image generation API request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Image generation API error (HTTP {}): {}", status, error_text));
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;
    
    // Try to extract image URL from different response formats
    let image_url = if config.provider == "dashscope" {
        // DashScope format
        json["output"]["results"][0]["url"]
            .as_str()
            .ok_or("Failed to extract image URL from DashScope response")?
            .to_string()
    } else {
        // OpenAI-compatible format - try URL first, then base64
        if let Some(url) = json["data"][0]["url"].as_str() {
            url.to_string()
        } else if let Some(b64) = json["data"][0]["b64_json"].as_str() {
            // Handle base64 response - decode and save directly
            let image_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
                .map_err(|e| format!("Failed to decode base64 image: {}", e))?;
            tokio::fs::write(output_path, &image_bytes)
                .await
                .map_err(|e| format!("Failed to save image: {}", e))?;
            return Ok(());
        } else {
            return Err(format!("Failed to extract image from response. Response keys: {:?}", 
                json.as_object().map(|m| m.keys().collect::<Vec<_>>())));
        }
    };
    
    // Download and save the image
    let image_response = reqwest::get(image_url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;
    
    let image_bytes = image_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image data: {}", e))?;
    
    tokio::fs::write(output_path, &image_bytes)
        .await
        .map_err(|e| format!("Failed to save image: {}", e))?;
    
    Ok(())
}

/// Generate image with template as reference (using /images/edits endpoint for GPT-Image)
async fn generate_image_with_template(
    prompt: &str,
    width: u32,
    height: u32,
    config: &ImageConfig,
    output_path: &PathBuf,
    template_path: &PathBuf,
) -> Result<(), String> {
    // Read template image
    let template_data = tokio::fs::read(template_path)
        .await
        .map_err(|e| format!("Failed to read template image: {}", e))?;
    
    let model_lower = config.model.to_lowercase();
    let is_gpt_image = model_lower.contains("gpt");
    let is_agnes = model_lower.starts_with("agnes");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    if is_gpt_image {
        // GPT-Image: use /images/edits endpoint with multipart/form-data
        let edit_endpoint = config.endpoint.replace("/generations", "/edits");
        
        // Build multipart form
        let form = reqwest::multipart::Form::new()
            .text("model", config.model.clone())
            .text("prompt", prompt.to_string())
            .text("size", format!("{}x{}", width, height))
            .text("n", "1")
            .part("image[]", reqwest::multipart::Part::bytes(template_data)
                .file_name("template.png")
                .mime_str("image/png")
                .map_err(|e| format!("Failed to set mime type: {}", e))?);
        
        let mut request = client
            .post(&edit_endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key));
        
        // Add extra headers
        for header in &config.extra_headers {
            request = request.header(&header.key, &header.value);
        }
        
        let response = request
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Image generation API request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Image generation API error (HTTP {}): {}", status, error_text));
        }
        
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;
        
        // Handle response
        handle_image_response(&json, output_path).await?;
        Ok(())
    } else if is_agnes {
        // Agnes models: use extra_body.image parameter with data URI
        let base64_template = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &template_data);
        let data_uri = format!("data:image/png;base64,{}", base64_template);
        
        let request_body = serde_json::json!({
            "model": config.model,
            "prompt": prompt,
            "size": format!("{}x{}", width, height),
            "extra_body": {
                "image": [data_uri],
                "response_format": "b64_json"
            }
        });
        
        let mut request = client
            .post(&config.endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json");
        
        // Add extra headers
        for header in &config.extra_headers {
            request = request.header(&header.key, &header.value);
        }
        
        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Image generation API request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Image generation API error (HTTP {}): {}", status, error_text));
        }
        
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;
        
        // Handle response
        handle_image_response(&json, output_path).await?;
        Ok(())
    } else {
        // Other APIs: use /images/generations with JSON body
        let base64_template = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &template_data);
        let data_uri = format!("data:image/png;base64,{}", base64_template);
        
        let request_body = serde_json::json!({
            "model": config.model,
            "prompt": prompt,
            "image": [data_uri],
            "size": format!("{}x{}", width, height),
            "n": 1,
        });
        
        let mut request = client
            .post(&config.endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json");
        
        // Add extra headers
        for header in &config.extra_headers {
            request = request.header(&header.key, &header.value);
        }
        
        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Image generation API request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Image generation API error (HTTP {}): {}", status, error_text));
        }
        
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;
        
        // Handle response
        handle_image_response(&json, output_path).await?;
        Ok(())
    }
}

/// Handle image response from API (URL or base64)
async fn handle_image_response(json: &serde_json::Value, output_path: &PathBuf) -> Result<(), String> {
    // Try URL first, then base64
    let image_url = if let Some(url) = json["data"][0]["url"].as_str() {
        url.to_string()
    } else if let Some(b64) = json["data"][0]["b64_json"].as_str() {
        // Handle base64 response - decode and save directly
        let image_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
            .map_err(|e| format!("Failed to decode base64 image: {}", e))?;
        tokio::fs::write(output_path, &image_bytes)
            .await
            .map_err(|e| format!("Failed to save image: {}", e))?;
        return Ok(());
    } else {
        return Err(format!("Failed to extract image from response. Response keys: {:?}", 
            json.as_object().map(|m| m.keys().collect::<Vec<_>>())));
    };
    
    // Download and save the image
    let image_response = reqwest::get(&image_url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;
    
    let image_bytes = image_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image data: {}", e))?;
    
    tokio::fs::write(output_path, &image_bytes)
        .await
        .map_err(|e| format!("Failed to save image: {}", e))?;
    
    Ok(())
}
