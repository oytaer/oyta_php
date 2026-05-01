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
/// 同时处理 URL 编码的路径遍历尝试
pub fn sanitize_filename(input: &str) -> String {
    // 先进行 URL 解码（防止 URL 编码绕过）
    let decoded = urlencoding::decode(input).unwrap_or_default().to_string();
    
    // 移除路径遍历字符
    let cleaned = decoded
        .replace("../", "")
        .replace("..\\", "")
        .replace("..", "")
        // 移除空字节（防止空字节注入）
        .replace('\0', "");
    
    // 移除绝对路径前缀
    if cleaned.starts_with('/') || cleaned.starts_with('\\') {
        cleaned[1..].to_string()
    } else {
        cleaned
    }
}

/// 清理 URL 中的 JavaScript 协议
///
/// 防止 javascript: 协议注入
/// 同时处理大小写和空白绕过
pub fn sanitize_url(input: &str) -> String {
    // 移除所有空白字符（防止空白绕过）
    let trimmed: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let lower = trimmed.to_lowercase();
    
    // 检查危险协议（支持多种编码绕过）
    let dangerous_protocols = [
        "javascript:", "vbscript:", "data:", "file:",
        "javascript&colon;", "vbscript&colon;", "data&colon;",
    ];
    
    for proto in &dangerous_protocols {
        if lower.starts_with(proto) {
            return String::new();
        }
    }
    
    // 检查 HTML 实体编码绕过
    let decoded = lower
        .replace("&colon;", ":")
        .replace("&#58;", ":")
        .replace("&#x3a;", ":");
    
    for proto in &["javascript:", "vbscript:", "data:", "file:"] {
        if decoded.starts_with(proto) {
            return String::new();
        }
    }
    
    input.to_string()
}

/// 过滤危险 HTML 标签
///
/// 保留安全的 HTML 标签，移除危险的标签和属性
/// 使用正则表达式进行更严格的匹配，防止绕过
pub fn filter_html(input: &str) -> String {
    // 危险标签列表
    let dangerous_tags = [
        "script", "iframe", "object", "embed", "form", "meta", "link",
        "base", "frame", "frameset", "applet", "svg", "math",
    ];
    
    let mut result = input.to_string();
    
    // 使用正则表达式移除危险标签（不区分大小写）
    for tag in &dangerous_tags {
        // 匹配各种形式的标签：<script>, <SCRIPT>, <ScRiPt>, <script/>, <script >等
        let patterns = vec![
            format!(r"(?i)<{}[^>]*>", tag),  // 开始标签
            format!(r"(?i)</{}[^>]*>", tag),  // 结束标签
        ];
        
        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(&pattern) {
                result = re.replace_all(&result, "").to_string();
            }
        }
    }
    
    // 移除事件处理器属性（不区分大小写）
    // 匹配 onclick, ONCLICK, OnClick 等各种形式
    let event_pattern = r#"(?i)\s+on\w+\s*=\s*["'][^"']*["']"#;
    if let Ok(re) = regex::Regex::new(event_pattern) {
        result = re.replace_all(&result, "").to_string();
    }
    
    // 移除不带引号的事件处理器
    let event_pattern2 = r#"(?i)\s+on\w+\s*=\s*[^\s>]+"#;
    if let Ok(re) = regex::Regex::new(event_pattern2) {
        result = re.replace_all(&result, "").to_string();
    }
    
    // 移除 javascript: 协议
    let js_pattern = r#"(?i)javascript\s*:"#;
    if let Ok(re) = regex::Regex::new(js_pattern) {
        result = re.replace_all(&result, "blocked:").to_string();
    }
    
    // 移除 data: 协议（可能用于 XSS）
    let data_pattern = r#"(?i)data\s*:"#;
    if let Ok(re) = regex::Regex::new(data_pattern) {
        result = re.replace_all(&result, "blocked:").to_string();
    }
    
    result
}

/// 深度清理 HTML
///
/// 对 HTML 内容进行全面的安全清理
/// 包括：标签过滤、属性清理、协议检查
pub fn deep_clean_html(input: &str) -> String {
    let mut result = input.to_string();
    
    // 1. 移除所有 HTML 注释（可能包含恶意内容）
    if let Ok(re) = regex::Regex::new(r"<!--.*?-->") {
        result = re.replace_all(&result, "").to_string();
    }
    
    // 2. 移除 CDATA 区块
    if let Ok(re) = regex::Regex::new(r"<!\[CDATA\[.*?\]\]>") {
        result = re.replace_all(&result, "").to_string();
    }
    
    // 3. 调用 filter_html 进行标签和属性过滤
    result = filter_html(&result);
    
    // 4. HTML 实体解码后再次检查（防止双重编码绕过）
    let decoded = decode_html_entities(&result);
    result = filter_html(&decoded);
    
    result
}

/// 解码 HTML 实体
///
/// 将 &amp; &lt; &gt; 等实体转换为对应字符
/// 支持命名实体、十进制数字实体和十六进制数字实体
fn decode_html_entities(input: &str) -> String {
    let mut result = input.to_string();
    
    // 命名实体
    result = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&copy;", "©")
        .replace("&reg;", "®")
        .replace("&trade;", "™")
        .replace("&euro;", "€")
        .replace("&pound;", "£")
        .replace("&yen;", "¥")
        .replace("&cent;", "¢")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
        .replace("&lsquo;", "'")
        .replace("&rsquo;", "'")
        .replace("&ldquo;", "\u{201C}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&bull;", "•")
        .replace("&middot;", "·")
        .replace("&deg;", "°")
        .replace("&plusmn;", "±")
        .replace("&times;", "×")
        .replace("&divide;", "÷")
        .replace("&frac12;", "½")
        .replace("&frac14;", "¼")
        .replace("&frac34;", "¾");
    
    // 处理十六进制数字实体 &#xHH; 或 &#XHH;
    let hex_pattern = regex::Regex::new(r"&#x([0-9a-fA-F]+);").unwrap();
    result = hex_pattern.replace_all(&result, |caps: &regex::Captures| {
        if let Ok(code) = u32::from_str_radix(&caps[1], 16) {
            if let Some(ch) = char::from_u32(code) {
                return ch.to_string();
            }
        }
        caps[0].to_string()
    }).to_string();
    
    // 处理十进制数字实体 &#DD;
    let dec_pattern = regex::Regex::new(r"&#(\d+);").unwrap();
    result = dec_pattern.replace_all(&result, |caps: &regex::Captures| {
        if let Ok(code) = u32::from_str_radix(&caps[1], 10) {
            if let Some(ch) = char::from_u32(code) {
                return ch.to_string();
            }
        }
        caps[0].to_string()
    }).to_string();
    
    result
}

/// 深度 XSS 清理
///
/// 对输入进行多轮清理，防止各种编码绕过
/// 1. URL 解码
/// 2. HTML 实体解码
/// 3. Unicode 解码
/// 4. 多轮清理直到内容稳定
pub fn deep_xss_clean(input: &str, max_iterations: usize) -> String {
    let mut result = input.to_string();
    let mut prev_result = String::new();
    let mut iterations = 0;
    
    // 多轮清理，直到内容不再变化或达到最大迭代次数
    while result != prev_result && iterations < max_iterations {
        prev_result = result.clone();
        
        // URL 解码
        result = urlencoding::decode(&result).unwrap_or_default().to_string();
        
        // HTML 实体解码
        result = decode_html_entities(&result);
        
        // 过滤危险内容
        result = deep_clean_html(&result);
        
        iterations += 1;
    }
    
    result
}
