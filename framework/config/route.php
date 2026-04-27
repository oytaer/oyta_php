<?php

/**
 * OYTAPHP 路由配置文件
 * 
 * 配置URL路由规则和路由模式
 * 
 * 路由模式：
 * - 普通模式：自动路由，如 /index/hello 映射到 IndexController::hello
 * - 混合模式：同时支持自动路由和定义路由
 * - 强制模式：必须定义路由，未定义的路由返回404
 * 
 * 使用示例：
 * // 在 route/route.php 中定义路由
 * Route::get('hello/:name', 'index/hello');
 * Route::post('user', 'user/save');
 */

return [
    // 是否强制路由
    // true: 必须定义路由才能访问
    // false: 支持自动路由
    'url_route_must' => false,
    
    // URL是否完全匹配
    'url_complete_match' => false,
    
    // URL是否自动转换（如驼峰转下划线）
    'url_convert' => true,
    
    // 路由是否延迟加载
    'url_lazy_load' => false,
    
    // 是否启用注解路由
    'route_annotation' => true,
    
    // 路由是否完全匹配
    'route_complete_match' => false,
    
    // 是否合并路由正则规则
    'route_merge_rule_regex' => false,
    
    // 是否检查路由缓存
    'route_check_cache' => false,
    
    // 路由缓存键名
    'route_cache_key' => 'route_cache',
    
    // 是否启用域名部署
    'url_domain_deploy' => false,
    
    // URL参数是否使用普通模式
    'url_common_param' => false,
    
    // URL参数类型
    'url_param_type' => 0,
    
    // 控制器层名称
    'url_controller_layer' => 'controller',
    
    // PATHINFO变量名（用于兼容模式）
    'var_pathinfo' => 's',
    
    // 请求方法伪装变量名
    'var_method' => '_method',
    
    // AJAX请求标识变量名
    'var_ajax' => '_ajax',
    
    // PJAX请求标识变量名
    'var_pjax' => '_pjax',
    
    // PATHINFO获取顺序
    'pathinfo_fetch' => ['ORIG_PATH_INFO', 'REDIRECT_PATH_INFO', 'REDIRECT_URL'],
    
    // URL伪静态后缀
    'url_html_suffix' => 'html',
    
    // 是否开启路由
    'url_route_on' => true,
];
