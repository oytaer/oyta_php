//! XSS 防护模块
//!
//! 提供跨站脚本攻击（XSS）防护功能
//! 包括 HTML 实体转义、JavaScript 转义、URL 编码、CSS 转义
//! 兼容 ThinkPHP 8.0 的安全过滤接口

/// HTML 实体转义
///
/// 将特殊字符转换为 HTML 实体，防止 XSS 攻击
/// 转义规则：
/// - & → &amp;
/// - < → &lt;
/// - > → &gt;
/// - " → &quot;
/// - ' → &#x27;
/// - / → &#x2F;
///
/// # 参数
/// - `input`: 待转义的字符串
///
/// # 返回
/// 转义后的安全字符串
pub fn escape_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            '/' => result.push_str("&#x2F;"),
            _ => result.push(ch),
        }
    }
    result
}

/// JavaScript 字符串转义
///
/// 防止在 JavaScript 上下文中注入恶意代码
/// 转义规则：
/// - \ → \\
/// - ' → \'
/// - " → \"
/// - 换行 → \n
/// - 回车 → \r
/// - 行分隔符 → \u2028
/// - 段落分隔符 → \u2029
///
/// # 参数
/// - `input`: 待转义的字符串
///
/// # 返回
/// 转义后的安全字符串
pub fn escape_js(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '\'' => result.push_str("\\'"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{2028}' => result.push_str("\\u2028"),
            '\u{2029}' => result.push_str("\\u2029"),
            _ => result.push(ch),
        }
    }
    result
}

/// CSS 字符串转义
///
/// 防止在 CSS 上下文中注入恶意代码
/// 非字母数字字符统一转换为 \HHHH 格式
///
/// # 参数
/// - `input`: 待转义的字符串
///
/// # 返回
/// 转义后的安全字符串
pub fn escape_css(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 6);
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            result.push(ch);
        } else {
            result.push_str(&format!("\\{:04X}", ch as u32));
        }
    }
    result
}

/// URL 编码
///
/// 对 URL 中的特殊字符进行编码
/// 使用 urlencoding crate 进行标准编码
pub fn escape_url(input: &str) -> String {
    urlencoding::encode(input).to_string()
}

/// 清理 HTML 标签
///
/// 移除所有 HTML 标签，只保留纯文本内容
///
/// # 参数
/// - `input`: 包含 HTML 标签的字符串
///
/// # 返回
/// 去除标签后的纯文本
pub fn strip_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(ch);
                }
            }
        }
    }
    result
}

/// 安全的 HTML 属性值
///
/// 对 HTML 属性值进行双重转义
/// 防止属性值中的 XSS 攻击
pub fn escape_attr(input: &str) -> String {
    escape_html(input)
}

/// 清理文件名中的路径遍历字符
///
/// 防止目录遍历攻击
/// 移除 ../ 和绝对路径前缀
pub fn sanitize_filename(input: &str) -> String {
    let cleaned = input
        .replace("../", "")
        .replace("..\\", "")
        .replace("..", "");
    if cleaned.starts_with('/') || cleaned.starts_with('\\') {
        cleaned[1..].to_string()
    } else {
        cleaned
    }
}

/// 清理 URL 中的 JavaScript 协议
///
/// 防止 javascript: 协议注入
pub fn sanitize_url(input: &str) -> String {
    let trimmed = input.trim().to_lowercase();
    if trimmed.starts_with("javascript:") || trimmed.starts_with("data:") || trimmed.starts_with("vbscript:") {
        String::new()
    } else {
        input.to_string()
    }
}

/// 过滤危险 HTML 标签
///
/// 保留安全的 HTML 标签，移除危险的标签和属性
/// 危险标签：script, iframe, object, embed, form, meta, link
pub fn filter_html(input: &str) -> String {
    let dangerous_tags = [
        "script", "iframe", "object", "embed", "form", "meta", "link",
        "base", "frame", "frameset", "applet",
    ];
    let mut result = input.to_string();
    for tag in &dangerous_tags {
        // 移除开始标签
        let open_pattern = format!("<{} ", tag);
        let open_pattern2 = format!("<{}>", tag);
        let close_pattern = format!("</{}>", tag);
        result = result.replace(&open_pattern, "&lt;filtered ");
        result = result.replace(&open_pattern2, "&lt;filtered&gt;");
        result = result.replace(&close_pattern, "&lt;/filtered&gt;");
    }
    // 移除事件处理器属性
    let event_attrs = [
        "onclick", "ondblclick", "onmousedown", "onmouseup",
        "onmouseover", "onmousemove", "onmouseout",
        "onkeydown", "onkeypress", "onkeyup",
        "onload", "onerror", "onsubmit", "onreset",
        "onchange", "onfocus", "onblur",
    ];
    for attr in &event_attrs {
        let patterns = [
            format!(" {}=", attr),
            format!(" {} =", attr),
        ];
        for pattern in &patterns {
            result = result.replace(pattern, " data-filtered=");
        }
    }
    result
}
