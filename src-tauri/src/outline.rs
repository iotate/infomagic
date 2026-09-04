use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::config::ApiConfig;
use crate::error_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageOutline {
    pub page_num: u32,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlineMode {
    Simple,   // 3-5 pages
    Medium,   // 6-10 pages
    Detailed, // 10-15 pages
}

impl OutlineMode {
    pub fn page_range(&self) -> (u32, u32) {
        match self {
            OutlineMode::Simple => (3, 5),
            OutlineMode::Medium => (6, 10),
            OutlineMode::Detailed => (10, 15),
        }
    }
    
    pub fn to_chinese(&self) -> &str {
        match self {
            OutlineMode::Simple => "简单",
            OutlineMode::Medium => "中等",
            OutlineMode::Detailed => "详细",
        }
    }
}

fn get_projects_dir(cwd: &PathBuf) -> PathBuf {
    cwd.join("projects")
}

#[tauri::command]
pub async fn generate_outline(
    cwd: State<'_, Arc<PathBuf>>,
    topic: String,
    mode: String,
    expected_pages: Option<u32>,
    style_name: Option<String>,
    config: ApiConfig,
) -> Result<String, String> {
    let cwd_path = cwd.inner().clone();
    
    let (min_pages, max_pages) = if let Some(pages) = expected_pages {
        // 自定义模式：使用用户指定的页数，允许 1-50 页
        let pages = pages.clamp(1, 50);
        (pages, pages)
    } else {
        // 预设模式
        let outline_mode = match mode.as_str() {
            "simple" => OutlineMode::Simple,
            "detailed" => OutlineMode::Detailed,
            _ => OutlineMode::Medium,
        };
        outline_mode.page_range()
    };
    
    // 读取风格内容
    let style_content = if let Some(name) = &style_name {
        let style_path = cwd.join("styles").join(format!("{}.md", name));
        if style_path.exists() {
            tokio::fs::read_to_string(&style_path).await.unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    // 读取风格内容并提取关键信息
    let (style_prompt, style_requirements) = if !style_content.is_empty() {
        // 提取风格提示词部分
        let style_prompt_section = if style_content.contains("## 风格提示词") {
            let start = style_content.find("## 风格提示词").unwrap_or(0);
            let end = style_content.find("## 核心要求").unwrap_or_else(|| style_content.find("## 适用").unwrap_or(style_content.len()));
            style_content[start..end].replace("## 风格提示词", "").trim().to_string()
        } else {
            style_content.clone()
        };
        
        // 构建每页必须包含的风格设计字段
        let style_design_field = format!(
            "\n**风格设计**: 全页统一遵循以下风格要求：\n{}\n", 
            style_prompt_section
        );
        
        (style_prompt_section, style_design_field)
    } else {
        (String::new(), String::new())
    };
    
    // 构建风格要求说明
    let style_instruction = if !style_prompt.is_empty() {
        format!("\n【重要】风格一致性要求：
所有页面必须严格遵循以下风格要求，不得偏离：
{}

每页的「风格设计」字段必须包含上述风格的具体应用说明。
", style_prompt)
    } else {
        String::new()
    };
    
    // 全局布局规范 - 会被写入每页内容
    let layout_spec = r#"
【全局布局规范 - 必须出现在每页内容中】
每个页面的 markdown 必须包含以下布局规范字段：

**布局设计**: 
- Logo位置：左上角，高度为画面高度3%-5%
- 页面标题：顶部区域，占画面高度8%-12%，字号24-32pt
- 内容区域：中部区域，占画面高度70%-80%
- 页码位置：右下角，字号12-14pt
- 留白要求：四周留白不少于画面边缘5%
- 模块间距：2%-4%画面高度
"#;
    
    let prompt = format!(
        r#"请为以下主题生成一个信息图表大纲，包含{}到{}页。
{}
{}

主题：{}

请按以下格式输出（使用Markdown）：

# 主题：[主题名称]

---

## 第1页：封面

**标题**: [主标题]
**页面内容**: 
- 主体：[核心主题名称/Logo]
- 场景：[使用场景、受众语境]
- 核心信息：[副标题、日期、作者等补充信息]
**布局设计**: 
- Logo位置：左上角，高度为画面高度3%-5%
- 主标题位置：画面中央，占画面面积40%-60%
- 副标题位置：主标题下方，占画面高度10%-15%
- 页码位置：右下角，字号12-14pt
- 留白要求：四周留白不少于画面边缘5%
**风格设计**: [本页风格关键词及具体应用]
{}

---

## 第2页：[页面标题]

**标题**: [页面标题]
**页面内容**: 
- 主体：[本页核心主题]
- 场景：[背景/语境]
- 构图：[模块布局、层级关系、图形设计]
- 文本：[必须显示的标题、标签、关键数据]
- 细节：[图标、装饰元素、信息标注]
- 核心约束：[3-5个模块、信息流方向]
**布局设计**: 
- Logo位置：左上角，高度为画面高度3%-5%
- 页面标题：顶部区域，占画面高度8%-12%，字号24-32pt
- 内容区域：中部区域，占画面高度70%-80%，分为3-5个模块
- 模块间距：2%-4%画面高度
- 页码位置：右下角，字号12-14pt
- 留白要求：四周留白不少于画面边缘5%
**风格设计**: [本页风格关键词及具体应用]
{}

---

... (更多页面，格式同第2页)

## 第N页：封底

**标题**: 谢谢
**页面内容**: 
- 主体：[致谢语/口号/品牌标识]
- 联系方式：[电话/邮箱]
**布局设计**: 
- Logo位置：左上角，高度为画面高度3%-5%
- 致谢语位置：画面中央，占画面高度15%-20%
- 联系方式位置：画面底部1/3区域，居中排列
- 页码位置：右下角，字号12-14pt
- 留白要求：四周留白不少于画面边缘10%
**风格设计**: [本页风格关键词及具体应用]
{}

信息图设计原则：
1. 每页内容控制在 3-8 个模块
2. 使用色块、箭头、图标和留白控制复杂度
3. 避免长段正文，用简洁短句呈现
4. 数字信息要醒目
5. 第一页是封面，最后一页是封底
6. 内容页之间用 --- 分隔
7. 所有页面必须严格遵循全局布局规范，固定元素位置和大小保持一致
"#,
        min_pages, max_pages, style_instruction, layout_spec, topic, style_requirements, style_requirements, style_requirements
    );

    // Log the start of generation (只记录动作，不记录具体内容)
    error_log::log_info(&cwd_path, "Starting outline generation");

    // Call LLM API
    let client = reqwest::Client::new();
    
    let mut request = client
        .post(format!("{}/chat/completions", config.endpoint))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json");
    
    // Add extra headers
    for header in &config.extra_headers {
        request = request.header(&header.key, &header.value);
    }
    
    let response = request
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": "你是一个专业的信息图表设计师，擅长创建结构清晰、内容丰富的演示文稿大纲。"},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.7,
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                let error_text = resp.text().await.unwrap_or_default();
                let error_msg = format!("LLM API error: {}", error_text);
                error_log::log_error(&cwd_path, &error_msg);
                return Err(error_msg);
            }

            let json_result = resp.json::<serde_json::Value>().await;
            match json_result {
                Ok(json) => {
                    let content = json["choices"][0]["message"]["content"]
                        .as_str()
                        .ok_or("Failed to extract content from LLM response");

                    match content {
                        Ok(content) => {
                            error_log::log_info(&cwd_path, "Outline generation completed successfully");
                            Ok(content.to_string())
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            error_log::log_error(&cwd_path, &error_msg);
                            Err(error_msg)
                        }
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to parse LLM response: {}", e);
                    error_log::log_error(&cwd_path, &error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("LLM API request failed: {}", e);
            error_log::log_error(&cwd_path, &error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn parse_outline(content: String) -> Result<Vec<PageOutline>, String> {
    let pages: Vec<&str> = content.split("\n---\n").collect();
    let mut outlines = Vec::new();
    
    for (i, page_content) in pages.iter().enumerate() {
        let page_num = (i + 1) as u32;
        
        let title = extract_field(page_content, "标题");
        let content = extract_field(page_content, "页面内容");
        
        outlines.push(PageOutline {
            page_num,
            title,
            content,
        });
    }
    
    Ok(outlines)
}

fn extract_field(content: &str, field_name: &str) -> String {
    for line in content.lines() {
        if line.starts_with(&format!("**{}**", field_name)) {
            return line
                .split(':')
                .skip(1)
                .collect::<Vec<_>>()
                .join(":")
                .trim()
                .to_string();
        }
    }
    String::new()
}

#[tauri::command]
pub async fn save_outline(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    content: String,
) -> Result<(), String> {
    let project_dir = get_projects_dir(&cwd).join(&project_name);
    let outline_path = project_dir.join("outline.md");
    
    tokio::fs::write(&outline_path, content)
        .await
        .map_err(|e| format!("Failed to save outline: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn load_prompt(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
) -> Result<String, String> {
    let project_dir = get_projects_dir(&cwd).join(&project_name);
    let prompt_path = project_dir.join("prompt.md");
    
    if !prompt_path.exists() {
        return Ok(String::new());
    }
    
    let content = tokio::fs::read_to_string(&prompt_path)
        .await
        .map_err(|e| format!("Failed to read prompt: {}", e))?;
    
    Ok(content)
}

#[tauri::command]
pub async fn save_prompt(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    content: String,
) -> Result<(), String> {
    let project_dir = get_projects_dir(&cwd).join(&project_name);
    let prompt_path = project_dir.join("prompt.md");
    
    tokio::fs::write(&prompt_path, content)
        .await
        .map_err(|e| format!("Failed to save prompt: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn regenerate_page(
    cwd: State<'_, Arc<PathBuf>>,
    project_name: String,
    page_num: u32,
    prompt: String,
    config: ApiConfig,
) -> Result<String, String> {
    let project_dir = get_projects_dir(&cwd).join(&project_name);
    let outline_path = project_dir.join("outline.md");
    
    let content = tokio::fs::read_to_string(&outline_path)
        .await
        .map_err(|e| format!("Failed to read outline: {}", e))?;
    
    let pages: Vec<&str> = content.split("\n---\n").collect();
    let page_index = (page_num - 1) as usize;
    
    if page_index >= pages.len() {
        return Err(format!("Page {} not found", page_num));
    }
    
    let current_page = pages[page_index];
    
    let system_prompt = "你是一个专业的信息图表设计师，擅长优化和改进演示文稿内容。";
    let user_prompt = format!(
        "请根据以下要求重新生成这一页的内容：\n\n原始内容：\n{}\n\n修改要求：{}\n\n请保持原有格式输出。",
        current_page, prompt
    );
    
    let client = reqwest::Client::new();
    
    let mut request = client
        .post(format!("{}/chat/completions", config.endpoint))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json");
    
    // Add extra headers
    for header in &config.extra_headers {
        request = request.header(&header.key, &header.value);
    }
    
    let response = request
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.7,
        }))
        .send()
        .await
        .map_err(|e| format!("LLM API request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("LLM API error: {}", error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let new_content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Failed to extract content from LLM response")?
        .to_string();
    
    // Update the page in outline
    let mut pages_vec: Vec<String> = pages.iter().map(|s| s.to_string()).collect();
    pages_vec[page_index] = new_content.clone();
    
    let updated_content = pages_vec.join("\n---\n");
    tokio::fs::write(&outline_path, updated_content)
        .await
        .map_err(|e| format!("Failed to update outline: {}", e))?;
    
    Ok(new_content)
}
