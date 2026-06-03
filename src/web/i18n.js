/**
 * PyForge Web i18n 模块
 *
 * 提供中英文翻译、语言检测、DOM 替换和语言切换功能。
 * 用法：在页面 <script> 中调用 initI18n() 初始化。
 */
(function(global){
  var translations = {
    // 通用
    'nav.dashboard':      { zh: '仪表盘', en: 'Dashboard' },
    'nav.projects':       { zh: '项目', en: 'Projects' },
    'nav.dependencies':   { zh: '依赖分析', en: 'Dependencies' },
    'theme.light':        { zh: '亮色', en: 'Light' },
    'theme.dark':         { zh: '暗色', en: 'Dark' },
    'lang.label':         { zh: '中文', en: 'EN' },

    // Dashboard
    'dashboard.title':        { zh: 'PyForge 仪表盘', en: 'PyForge Dashboard' },
    'dashboard.status':       { zh: '服务运行中', en: 'Server is running' },
    'dashboard.loading':      { zh: '加载中...', en: 'Loading...' },
    'dashboard.total':        { zh: '项目总数', en: 'Total Projects' },
    'dashboard.recent':       { zh: '近7天活跃', en: 'Recent Active (7d)' },
    'dashboard.version_dist': { zh: 'Python 版本分布', en: 'Python Version Distribution' },
    'dashboard.toolchain_dist': { zh: '工具链分布', en: 'Toolchain Distribution' },
    'dashboard.recent_list':  { zh: '近期活跃项目', en: 'Recent Active Projects' },
    'dashboard.no_recent':    { zh: '近 7 天内无项目修改。', en: 'No projects modified in the last 7 days.' },
    'dashboard.error_stats':  { zh: '加载统计失败：', en: 'Failed to load stats: ' },
    'dashboard.no_data':      { zh: '暂无数据', en: 'No data' },

    // Projects
    'projects.title':         { zh: '项目列表', en: 'Projects' },
    'projects.search':        { zh: '按名称或路径搜索...', en: 'Search by name or path...' },
    'projects.all_toolchains': { zh: '所有工具链', en: 'All Toolchains' },
    'projects.count':         { zh: '{n} 个项目', en: '{n} project(s)' },
    'projects.empty':         { zh: '未找到项目。', en: 'No projects found.' },
    'projects.error':         { zh: '加载项目失败：', en: 'Failed to load projects: ' },
    'projects.no_git':        { zh: '无 git', en: 'no git' },
    'projects.clean':         { zh: '干净', en: 'clean' },

    // Project Detail
    'detail.loading':         { zh: '加载中...', en: 'Loading...' },
    'detail.loading_detail':  { zh: '正在加载项目详情...', en: 'Loading project details...' },
    'detail.error':           { zh: '未找到项目', en: 'Project not found' },
    'detail.open_dir':        { zh: '打开目录', en: 'Open Directory' },
    'detail.open_terminal':   { zh: '打开终端', en: 'Open Terminal' },
    'detail.open_vscode':     { zh: '用 VS Code 打开', en: 'Open in VS Code' },
    'detail.info':            { zh: '项目信息', en: 'Project Info' },
    'detail.name':            { zh: '名称', en: 'Name' },
    'detail.path':            { zh: '路径', en: 'Path' },
    'detail.python_version':  { zh: 'Python 版本', en: 'Python Version' },
    'detail.toolchain':       { zh: '工具链', en: 'Toolchain' },
    'detail.git_remote':      { zh: 'Git 远程', en: 'Git Remote' },
    'detail.git_branch':      { zh: 'Git 分支', en: 'Git Branch' },
    'detail.git_status':      { zh: 'Git 状态', en: 'Git Status' },
    'detail.deps_count':      { zh: '依赖数', en: 'Dependencies' },
    'detail.created':         { zh: '创建时间', en: 'Created' },
    'detail.modified':        { zh: '最后修改', en: 'Last Modified' },
    'detail.description':     { zh: '描述', en: 'Description' },
    'detail.tags':            { zh: '标签', en: 'Tags' },

    // Dependencies
    'deps.title':             { zh: '依赖分析', en: 'Dependencies' },
    'deps.total':             { zh: '依赖总数', en: 'Total Dependencies' },
    'deps.covered':           { zh: '覆盖项目', en: 'Projects Covered' },
    'deps.shared':            { zh: '共享依赖 (2+)', en: 'Shared (2+)' },
    'deps.cross_project':     { zh: '跨项目依赖', en: 'Cross-Project Dependencies' },
    'deps.search':            { zh: '搜索依赖...', en: 'Search dependencies...' },
    'deps.col_name':          { zh: '依赖名称', en: 'Dependency' },
    'deps.col_count':         { zh: '次数', en: 'Count' },
    'deps.col_projects':      { zh: '项目', en: 'Projects' },
    'deps.loading':           { zh: '加载中...', en: 'Loading...' },
    'deps.no_match':          { zh: '未找到匹配的依赖。', en: 'No matching dependencies found.' },
    'deps.empty':             { zh: '未找到依赖。', en: 'No dependencies found.' },
    'deps.error':             { zh: '加载依赖失败：', en: 'Failed to load dependencies: ' }
  };

  /** 获取当前语言：localStorage > navigator.language > 'zh' */
  function getLocale() {
    var saved = localStorage.getItem('pyforge_locale');
    if (saved === 'zh' || saved === 'en') return saved;
    var nav = (navigator.language || navigator.userLanguage || '').toLowerCase();
    if (nav.indexOf('zh') === 0) return 'zh';
    return 'en';
  }

  /** 翻译一个 key，支持 {n} 占位符 */
  function t(key, params) {
    var entry = translations[key];
    if (!entry) return key;
    var locale = getLocale();
    var text = entry[locale] || entry['zh'] || key;
    if (params) {
      for (var k in params) {
        if (params.hasOwnProperty(k)) {
          text = text.replace('{' + k + '}', params[k]);
        }
      }
    }
    return text;
  }

  /** 将翻译应用到 DOM 中所有 data-i18n 元素 */
  function applyI18n() {
    var locale = getLocale();
    document.documentElement.setAttribute('lang', locale);
    var elements = document.querySelectorAll('[data-i18n]');
    for (var i = 0; i < elements.length; i++) {
      var el = elements[i];
      var key = el.getAttribute('data-i18n');
      var entry = translations[key];
      if (entry) {
        var text = entry[locale] || entry['zh'] || key;
        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {
          el.placeholder = text;
        } else {
          el.textContent = text;
        }
      }
    }
    // 更新语言切换按钮文本
    var langBtn = document.getElementById('langToggle');
    if (langBtn) {
      langBtn.textContent = locale === 'zh' ? 'EN' : '中文';
    }
    // 更新主题按钮文本
    var themeBtn = document.getElementById('themeToggle');
    if (themeBtn) {
      var currentTheme = document.documentElement.getAttribute('data-theme') || 'dark';
      themeBtn.textContent = currentTheme === 'dark' ? t('theme.light') : t('theme.dark');
    }
  }

  /** 初始化 i18n：应用翻译 + 绑定语言切换按钮 */
  function initI18n() {
    applyI18n();
    var langBtn = document.getElementById('langToggle');
    if (langBtn) {
      langBtn.addEventListener('click', function() {
        var current = getLocale();
        var next = current === 'zh' ? 'en' : 'zh';
        localStorage.setItem('pyforge_locale', next);
        applyI18n();
        // 触发自定义事件，让页面的动态渲染逻辑可以重新渲染
        var evt = document.createEvent('Event');
        evt.initEvent('localeChanged', true, true);
        document.dispatchEvent(evt);
      });
    }
  }

  global.PyForgeI18n = {
    getLocale: getLocale,
    t: t,
    applyI18n: applyI18n,
    initI18n: initI18n
  };
})(window);
