//! OpenAI 兼容 LLM 层（支持工具调用）。llm-core 不支持 tools，所以这里自带一个最小客户端。

use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};

pub struct Model {
    http: reqwest::Client,
    base_url: String,
    model: String,
}

impl Model {
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("BASE_URL").context("缺少环境变量 BASE_URL")?;
        let api_key = std::env::var("OPENAI_API_KEY").context("缺少环境变量 OPENAI_API_KEY")?;
        let model = std::env::var("OPENAI_MODEL").context("缺少环境变量 OPENAI_MODEL")?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !api_key.is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {api_key}"))?,
            );
        }
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Self {
            http,
            base_url,
            model,
        })
    }

    /// 调用一次模型，返回 (assistant 消息 Value, Option<(输入 token, 输出 token)>)。
    pub async fn call(
        &self,
        messages: &[Value],
        tools: &[Value],
    ) -> Result<(Value, Option<(u64, u64)>)> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "messages": messages,
            "tools": tools,
            "tool_choice": "auto",
        });
        let resp: Value = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .context("请求失败")?
            .error_for_status()
            .context("模型端点返回错误")?
            .json()
            .await
            .context("解析响应 JSON 失败")?;

        let message = resp["choices"][0]["message"].clone();
        let usage = resp.get("usage").map(|u| {
            (
                u["prompt_tokens"].as_u64().unwrap_or(0),
                u["completion_tokens"].as_u64().unwrap_or(0),
            )
        });
        Ok((message, usage))
    }
}
