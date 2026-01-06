use eframe::egui;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::winnt::{TOKEN_QUERY, TOKEN_ELEVATION, TokenElevation};
use winapi::ctypes::c_void;
use winapi::um::winnt::HANDLE;
use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use std::ptr;

// 检查当前进程是否有管理员权限
fn is_admin() -> bool {
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        
        // 获取当前进程的句柄
        let process_handle = winapi::um::processthreadsapi::GetCurrentProcess();
        
        // 打开进程令牌
        if OpenProcessToken(
            process_handle,
            TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length: u32 = 0;

        // 获取令牌提升信息
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut c_void,
            std::mem::size_of_val(&elevation) as u32,
            &mut return_length,
        );

        result != 0 && elevation.TokenIsElevated != 0
    }
}


// 转换大小为字节
fn size_to_bytes(size: f64, unit: &str) -> u64 {
    let multiplier = match unit {
        "KB" => 1024.0,
        "MB" => 1024.0 * 1024.0,
        "GB" => 1024.0 * 1024.0 * 1024.0,
        _ => 1.0, // 默认字节
    };
    (size * multiplier) as u64
}

// 获取系统分辨率
fn get_system_resolution() -> (i32, i32) {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);
        (width, height)
    }
}

// 执行创建文件的命令
fn create_empty_file(path: &str, size_bytes: u64) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};
    
    // 尝试创建文件
    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
    {
        Ok(file) => file,
        Err(e) => return Err(format!("打开文件失败: {}", e)),
    };
    
    // 如果大小为0，直接返回
    if size_bytes == 0 {
        return Ok(());
    }
    
    // 设置文件大小
    match file.seek(SeekFrom::Start(size_bytes - 1)) {
        Ok(_) => {},
        Err(e) => return Err(format!("设置文件指针位置失败: {}", e)),
    }
    
    // 写入一个空字节，实际创建指定大小的文件
    match file.write_all(&[0]) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("写入文件失败: {}", e)),
    }
}

// 主界面逻辑
struct FileCreatorApp {
    save_dir: String,           // 保存目录
    file_size: String,         // 文件大小
    size_unit: String,         // 大小单位
    status_msg: String,        // 状态消息
}

impl Default for FileCreatorApp {
    fn default() -> Self {
        Self {
            save_dir: String::new(),
            file_size: "1".to_string(),
            size_unit: "MB".to_string(),
            status_msg: String::new(),
        }
    }
}

// FileCreatorApp的方法实现
impl FileCreatorApp {
    // 创建文件的方法
    fn create_file(&mut self) {
        // 输入验证
        if self.save_dir.is_empty() {
            self.status_msg = "❌ 错误：保存目录不能为空！".to_string();
            return;
        }

        let size = match self.file_size.parse::<f64>() {
            Ok(s) => s,
            Err(_) => {
                self.status_msg = "❌ 错误：文件大小必须是数字！".to_string();
                return;
            }
        };

        if size <= 0.0 {
            self.status_msg = "❌ 错误：文件大小必须大于0！".to_string();
            return;
        }

        // 生成文件名：xxx.devrom，例如 1MB.devrom 或 450GB.devrom
        let filename = format!("{}{}.devrom", self.file_size, self.size_unit);
        
        // 组合完整路径
        let full_path = std::path::Path::new(&self.save_dir).join(filename);
        let full_path_str = full_path.to_string_lossy().to_string();

        // 转换大小为字节
        let size_bytes = size_to_bytes(size, &self.size_unit);

        // 执行创建命令
        match create_empty_file(&full_path_str, size_bytes) {
            Ok(_) => {
                self.status_msg = format!(
                    "✅ 成功：已创建文件 {} (大小：{} {})",
                    full_path_str, self.file_size, self.size_unit
                );
            }
            Err(e) => {
                self.status_msg = format!("❌ 失败：{}", e);
            }
        }
    }
}

impl eframe::App for FileCreatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📁 空文件创建工具,另类快速鉴别扩容盘，项目地址：https://github.com/aspnmy/rust_file_creator");
            ui.heading("此工具github上开源免费如果你付费购买就是上当了");
            ui.heading("除了跑圈你还可以这样：向指定要测试的固态路径写入一个小于标称容量1GB的空文件，写入成功为足量，此方法无需等待跑圈");
            ui.heading("常见标称/足量之间关系：2TB/1800GB、1TB/890GB、512GB/450GB、256GB/225GB、128GB/112GB");
            ui.heading("空文件创建方案因为不提供测速，如需要测速你可以把文件创建再其他盘然后复制到需要测试的固态路径中，即可得到写入速率");

            ui.separator();

            // 权限检查提示
            if !is_admin() {
                ui.colored_label(egui::Color32::RED, "⚠️ 警告：当前无管理员权限，创建文件会失败！请以管理员身份运行本程序。");
                ui.separator();
            }

            // 保存目录选择
            ui.horizontal(|ui| {
                ui.label("保存目录:");
                ui.text_edit_singleline(&mut self.save_dir);
                
                // 优化目录选择体验
                if ui.button("浏览").clicked() {
                    // 使用rfd的构建器模式，链式调用设置属性
                    let path = rfd::FileDialog::new()
                        .set_title("选择保存目录")
                        // 设置起始目录，优化加载速度
                        .set_directory(
                            if !self.save_dir.is_empty() {
                                &self.save_dir
                            } else {
                                // 优先使用当前目录，避免复杂的目录查找
                                "."
                            }
                        )
                        .pick_folder();
                    
                    // 直接处理结果，避免额外的变量
                    if let Some(selected_path) = path {
                        self.save_dir = selected_path.to_string_lossy().to_string();
                    }
                }
            });
            
            // 文件名说明
            ui.label("📌 提示：文件名将根据选择的文件大小自动生成，格式为 xxx.devrom（例如：1MB.devrom 或 450GB.devrom）");

            // 文件大小设置
            ui.horizontal(|ui| {
                ui.label("文件大小:");
                ui.text_edit_singleline(&mut self.file_size);
                
                egui::ComboBox::from_label("单位")
                    .selected_text(&self.size_unit)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.size_unit, "B".to_string(), "字节 (B)");
                        ui.selectable_value(&mut self.size_unit, "KB".to_string(), "千字节 (KB)");
                        ui.selectable_value(&mut self.size_unit, "MB".to_string(), "兆字节 (MB)");
                        ui.selectable_value(&mut self.size_unit, "GB".to_string(), "吉字节 (GB)");
                    });
            });

            // 快捷大小按钮
            ui.horizontal(|ui| {
                ui.label("快捷创建大小:");
                
                // 常见容量快捷按钮
                if ui.button("1800GB (2TB)").clicked() {
                    self.file_size = "1800".to_string();
                    self.size_unit = "GB".to_string();
                    // 自动创建文件
                    self.create_file();
                }
                
                if ui.button("890GB (1TB)").clicked() {
                    self.file_size = "890".to_string();
                    self.size_unit = "GB".to_string();
                    // 自动创建文件
                    self.create_file();
                }
                
                if ui.button("450GB (512GB)").clicked() {
                    self.file_size = "450".to_string();
                    self.size_unit = "GB".to_string();
                    // 自动创建文件
                    self.create_file();
                }
            });
            
            // 第二行快捷按钮
            ui.horizontal(|ui| {
                // 继续添加快捷按钮
                if ui.button("225GB (256GB)").clicked() {
                    self.file_size = "225".to_string();
                    self.size_unit = "GB".to_string();
                    // 自动创建文件
                    self.create_file();
                }
                
                if ui.button("112GB (128GB)").clicked() {
                    self.file_size = "112".to_string();
                    self.size_unit = "GB".to_string();
                    // 自动创建文件
                    self.create_file();
                }
            });

            // 创建文件按钮
            if ui.button("创建空文件").clicked() {
                self.create_file();
            }

            ui.separator();
            // 状态提示
            ui.label(&self.status_msg);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    // 获取系统分辨率
    let (screen_width, screen_height) = get_system_resolution();
    
    // 根据系统分辨率计算合适的窗口大小 (50% 宽度, 40% 高度)
    let window_width = (screen_width as f32 * 0.5).max(600.0); // 最小宽度600
    let window_height = (screen_height as f32 * 0.4).max(350.0); // 最小高度350
    
    // 配置界面外观
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(window_width, window_height)),
        ..Default::default()
    };

    // 运行应用
    eframe::run_native(
        "QuickSSDChecker v0.1.1 (DIY 固态群:115405294)",
        native_options,
        Box::new(|cc| {
            // 配置字体，添加中文字体支持
            let mut fonts = egui::FontDefinitions::default();
            
            // 添加系统字体，Windows系统默认支持中文
            fonts.font_data.insert(
                "system_font".to_owned(),
                egui::FontData::from_static(include_bytes!(r"C:\Windows\Fonts\simhei.ttf")),
            );
            
            // 将系统字体添加到默认字体家族
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "system_font".to_owned());
            
            // 也添加到等宽字体家族，确保所有文本都能正确显示中文
            fonts.families.get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "system_font".to_owned());
            
            cc.egui_ctx.set_fonts(fonts);
            
            Box::new(FileCreatorApp::default())
        }),
    )
}