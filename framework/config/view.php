<?php

/**
 * OYTAPHP 视图配置文件
 * 
 * 使用Tera模板引擎，支持模板继承、布局、包含等特性
 * 模板语法兼容ThinkPHP风格
 * 
 * 使用示例：
 * // 控制器中渲染模板
 * return View::fetch('index/hello', [
 *     'name' => 'OYTAPHP',
 *     'title' => '欢迎页面',
 * ]);
 * 
 * // 模板中使用变量
 * {{ name }}
 * 
 * // 条件判断
 * {% if status == 1 %} 正常 {% endif %}
 * 
 * // 循环
 * {% for item in list %} {{ item.name }} {% endfor %}
 */

return [
    // 模板引擎类型
    'type' => 'Tera',
    
    // 视图路径（为空则使用默认路径 view/）
    'view_path' => '',
    
    // 视图文件后缀
    'view_suffix' => 'html',
    
    // 视图路径分隔符
    'view_depr' => '/',
    
    // 标签开始标记
    'tag_begin' => '{',
    
    // 标签结束标记
    'tag_end' => '}',
    
    // 标签库开始标记
    'taglib_begin' => '{',
    
    // 标签库结束标记
    'taglib_end' => '}',
    
    // 是否启用布局
    'layout_on' => false,
    
    // 布局模板名称
    'layout_name' => 'layout/index',
    
    // 布局内容替换标记
    'layout_item' => '{__CONTENT__}',
    
    // 模板缓存前缀
    'cache_prefix' => 'view_',
];
