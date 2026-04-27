<?php

/**
 * OYTAPHP 默认控制器
 * 
 * 本控制器演示了基本的控制器用法
 * 控制器命名空间为 app\controller
 * 
 * 访问方式：
 * - / 或 /index → index() 方法
 * - /index/hello → hello() 方法
 * - /index/hello/name/oyta → hello('oyta')
 * 
 * 注意：Controller 基类由 Rust 运行时提供，无需 require
 */

namespace app\controller;

use oyta\foundation\Controller;

class Index extends Controller
{
    /**
     * 默认首页
     * 
     * 访问路径：/ 或 /index/index
     * 
     * @return mixed 返回视图渲染结果
     */
    public function index()
    {
        return view('index/index', [
            'title' => 'OYTAPHP - Rust驱动的PHP框架',
            'message' => '欢迎使用 OYTAPHP！',
        ]);
    }

    /**
     * Hello示例方法
     * 
     * 访问路径：/index/hello 或 /index/hello/name/xxx
     * 
     * @param string $name 名称参数，默认值为 'OYTAPHP'
     * @return string 返回问候语
     */
    public function hello(string $name = 'OYTAPHP')
    {
        return 'Hello, ' . $name . '!';
    }
}
