using System;
using System.IO;
using System.IO.Compression;
using System.Text;
using System.Windows.Forms;

class EyeForgeInstaller
{
    [STAThread]
    static void Main()
    {
        Application.EnableVisualStyles();
        Application.Run(new InstallWizard());
    }
}

class InstallWizard : Form
{
    static readonly byte[] Marker = Encoding.ASCII.GetBytes("EYEFORGEZIP");

    private Panel[] _pages;
    private Button _prevBtn, _nextBtn, _cancelBtn;
    private int _currentPage = 0;
    private string _lang = "zh";

    private ComboBox _langCombo;
    private TextBox _pathBox;
    private CheckBox _autoDepCheck;
    private ProgressBar _progress;
    private Label _status;
    private CheckBox _shortcutCheck;
    private string _target;

    public InstallWizard()
    {
        Text = "EyeForge 安装 / Installation";
        Size = new System.Drawing.Size(560, 370);
        StartPosition = FormStartPosition.CenterScreen;
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        ShowInTaskbar = true;

        _pages = new Panel[5];
        _pages[0] = CreateLangPage();
        _pages[1] = CreateWelcomePage();
        _pages[2] = CreateFolderPage();
        _pages[3] = CreateInstallPage();
        _pages[4] = CreateFinishPage();

        for (int i = 0; i < _pages.Length; i++)
        {
            _pages[i].Visible = i == 0;
            _pages[i].SetBounds(0, 0, 554, 280);
            Controls.Add(_pages[i]);
        }

        _prevBtn = new Button();
        _prevBtn.Location = new System.Drawing.Point(140, 290);
        _prevBtn.Size = new System.Drawing.Size(120, 30);
        _prevBtn.Click += (s, e) => { ShowPage(_currentPage - 1); };
        Controls.Add(_prevBtn);

        _nextBtn = new Button();
        _nextBtn.Location = new System.Drawing.Point(270, 290);
        _nextBtn.Size = new System.Drawing.Size(120, 30);
        _nextBtn.Click += OnNext;
        Controls.Add(_nextBtn);

        _cancelBtn = new Button();
        _cancelBtn.Location = new System.Drawing.Point(410, 290);
        _cancelBtn.Size = new System.Drawing.Size(100, 30);
        _cancelBtn.Click += (s, e) => { Close(); };
        Controls.Add(_cancelBtn);

        UpdateUIText();
        ShowPage(0);
    }

    string T(string zh, string en) { return _lang == "en" ? en : zh; }

    Panel CreateWelcomePage()
    {
        Panel p = new Panel();
        p.AutoScroll = true;
        Label title = new Label();
        title.Text = "EyeForge v1.1";
        title.Font = new System.Drawing.Font("Microsoft YaHei", 16, System.Drawing.FontStyle.Bold);
        title.AutoSize = false;
        title.TextAlign = System.Drawing.ContentAlignment.MiddleCenter;
        title.Height = 50;
        title.Dock = DockStyle.Top;
        title.Padding = new Padding(0, 15, 0, 0);

        Label desc = new Label();
        desc.Text = "AI 屏幕操控助手\n\n"
            + "让大模型看到你的屏幕，并替你操作鼠标和键盘。\n\n"
            + "支持 OpenAI、Anthropic、Ollama、Gemini 及自定义 API。\n\n"
            + "---\n\n"
            + "AI Screen Control Assistant\n\n"
            + "Let AI models see your screen and control your mouse and keyboard.";
        desc.AutoSize = false;
        desc.TextAlign = System.Drawing.ContentAlignment.MiddleCenter;
        desc.Dock = DockStyle.Fill;
        desc.Padding = new Padding(30, 5, 30, 0);

        p.Controls.Add(desc);
        p.Controls.Add(title);
        return p;
    }

    Panel CreateLangPage()
    {
        Panel p = new Panel();

        Label label = new Label();
        label.Text = "选择安装程序语言\nSelect installer language:";
        label.AutoSize = false;
        label.TextAlign = System.Drawing.ContentAlignment.MiddleLeft;
        label.Location = new System.Drawing.Point(30, 50);
        label.Size = new System.Drawing.Size(500, 40);

        _langCombo = new ComboBox();
        _langCombo.Items.Add("中文");
        _langCombo.Items.Add("English");
        _langCombo.SelectedIndex = 0;
        _langCombo.Location = new System.Drawing.Point(30, 100);
        _langCombo.Size = new System.Drawing.Size(200, 28);
        _langCombo.SelectedIndexChanged += (s, e) =>
        {
            _lang = _langCombo.SelectedIndex == 1 ? "en" : "zh";
            UpdateUIText();
        };

        Label note = new Label();
        note.Tag = "lang_note";
        note.Text = "安装程序后续页面将使用所选语言\nSubsequent pages will use the selected language";
        note.ForeColor = System.Drawing.Color.Gray;
        note.Location = new System.Drawing.Point(30, 140);
        note.Size = new System.Drawing.Size(500, 30);

        p.Controls.Add(note);
        p.Controls.Add(_langCombo);
        p.Controls.Add(label);
        return p;
    }

    Panel CreateFolderPage()
    {
        Panel p = new Panel();

        Label label = new Label();
        label.Tag = "folder_label";
        label.Text = T("选择安装目录:", "Select installation directory:");
        label.AutoSize = false;
        label.TextAlign = System.Drawing.ContentAlignment.MiddleLeft;
        label.Location = new System.Drawing.Point(30, 30);
        label.Size = new System.Drawing.Size(500, 30);

        _pathBox = new TextBox();
        _pathBox.Text = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "EyeForge");
        _pathBox.Location = new System.Drawing.Point(30, 65);
        _pathBox.Size = new System.Drawing.Size(340, 24);

        Button browseBtn = new Button();
        browseBtn.Tag = "browse_btn";
        browseBtn.Text = T("浏览", "Browse");
        browseBtn.Location = new System.Drawing.Point(380, 63);
        browseBtn.Size = new System.Drawing.Size(90, 28);
        browseBtn.Click += (s, e) =>
        {
            FolderBrowserDialog fbd = new FolderBrowserDialog();
            fbd.SelectedPath = _pathBox.Text;
            fbd.ShowNewFolderButton = true;
            if (fbd.ShowDialog() == DialogResult.OK)
                _pathBox.Text = fbd.SelectedPath;
        };

        _autoDepCheck = new CheckBox();
        _autoDepCheck.Tag = "auto_dep_check";
        _autoDepCheck.Text = T("自动安装 Python 依赖（需要联网）",
                              "Auto-install Python dependencies (requires internet)");
        _autoDepCheck.Checked = true;
        _autoDepCheck.Location = new System.Drawing.Point(30, 110);
        _autoDepCheck.Size = new System.Drawing.Size(450, 30);

        Label note = new Label();
        note.Tag = "folder_note";
        note.Text = T("安装需要约 50MB 空间", "Requires ~50MB disk space");
        note.ForeColor = System.Drawing.Color.Gray;
        note.Location = new System.Drawing.Point(30, 150);
        note.Size = new System.Drawing.Size(500, 30);

        p.Controls.Add(note);
        p.Controls.Add(_autoDepCheck);
        p.Controls.Add(browseBtn);
        p.Controls.Add(_pathBox);
        p.Controls.Add(label);
        return p;
    }

    Panel CreateInstallPage()
    {
        Panel p = new Panel();

        _status = new Label();
        _status.Tag = "install_status";
        _status.Text = T("准备安装...", "Preparing...");
        _status.AutoSize = false;
        _status.TextAlign = System.Drawing.ContentAlignment.MiddleCenter;
        _status.Location = new System.Drawing.Point(30, 30);
        _status.Size = new System.Drawing.Size(500, 60);

        _progress = new ProgressBar();
        _progress.Style = ProgressBarStyle.Marquee;
        _progress.Location = new System.Drawing.Point(30, 110);
        _progress.Size = new System.Drawing.Size(500, 30);

        p.Controls.Add(_status);
        p.Controls.Add(_progress);
        return p;
    }

    Panel CreateFinishPage()
    {
        Panel p = new Panel();

        Label done = new Label();
        done.Tag = "finish_done";
        done.Text = T("安装完成！", "Installation complete!");
        done.Font = new System.Drawing.Font("Microsoft YaHei", 14, System.Drawing.FontStyle.Bold);
        done.AutoSize = false;
        done.TextAlign = System.Drawing.ContentAlignment.MiddleCenter;
        done.Location = new System.Drawing.Point(30, 40);
        done.Size = new System.Drawing.Size(500, 50);

        _shortcutCheck = new CheckBox();
        _shortcutCheck.Tag = "shortcut_check";
        _shortcutCheck.Text = T("创建桌面快捷方式", "Create desktop shortcut");
        _shortcutCheck.Checked = true;
        _shortcutCheck.Location = new System.Drawing.Point(30, 110);
        _shortcutCheck.Size = new System.Drawing.Size(400, 30);

        Label thanks = new Label();
        thanks.Tag = "finish_thanks";
        thanks.Text = T("感谢使用 EyeForge！", "Thank you for using EyeForge!");
        thanks.ForeColor = System.Drawing.Color.Gray;
        thanks.Location = new System.Drawing.Point(30, 150);
        thanks.Size = new System.Drawing.Size(500, 30);

        p.Controls.Add(thanks);
        p.Controls.Add(_shortcutCheck);
        p.Controls.Add(done);
        return p;
    }

    void UpdateUIText()
    {
        Text = T("EyeForge 安装", "EyeForge Installation");

        _prevBtn.Text = T("< 上一步", "< Back");
        _nextBtn.Text = _currentPage == 4 ? T("完成", "Finish") : T("下一步 >", "Next >");
        _cancelBtn.Text = T("取消", "Cancel");

        foreach (Control c in _pages[0].Controls)
        {
            if (c is Label && c.Tag == null) ((Label)c).Text = T("选择安装程序语言", "Select installer language:");
            if (c is Label && c.Tag != null && (string)c.Tag == "lang_note")
                ((Label)c).Text = T("安装程序后续页面将使用所选语言", "Subsequent pages will use the selected language");
        }

        foreach (Control c in _pages[2].Controls)
        {
            if (c is Label && (string)c.Tag == "folder_label")
                ((Label)c).Text = T("选择安装目录:", "Select installation directory:");
            if (c is Button && (string)c.Tag == "browse_btn")
                ((Button)c).Text = T("浏览", "Browse");
            if (c is CheckBox && (string)c.Tag == "auto_dep_check")
                ((CheckBox)c).Text = T("自动安装 Python 依赖（需要联网）",
                                       "Auto-install Python dependencies (requires internet)");
            if (c is Label && (string)c.Tag == "folder_note")
                ((Label)c).Text = T("安装需要约 50MB 空间", "Requires ~50MB disk space");
        }

        _status.Text = T("准备安装...", "Preparing...");

        foreach (Control c in _pages[4].Controls)
        {
            if (c is Label && (string)c.Tag == "finish_done")
                ((Label)c).Text = T("安装完成！", "Installation complete!");
            if (c is CheckBox && (string)c.Tag == "shortcut_check")
                ((CheckBox)c).Text = T("创建桌面快捷方式", "Create desktop shortcut");
            if (c is Label && (string)c.Tag == "finish_thanks")
                ((Label)c).Text = T("感谢使用 EyeForge！", "Thank you for using EyeForge!");
        }
    }

    void ShowPage(int idx)
    {
        _pages[_currentPage].Visible = false;
        _currentPage = idx;
        _pages[_currentPage].Visible = true;

        _prevBtn.Visible = idx > 0 && idx < 4;
        _prevBtn.Enabled = idx != 3;

        if (idx == 3)
        {
            _prevBtn.Enabled = false;
            _nextBtn.Enabled = false;
            _cancelBtn.Enabled = false;
        }
        else
        {
            _cancelBtn.Enabled = true;
            _nextBtn.Text = idx == 4 ? T("完成", "Finish") : T("下一步 >", "Next >");
        }
    }

    void OnNext(object sender, EventArgs e)
    {
        if (_currentPage == 0)
        {
            _lang = _langCombo.SelectedIndex == 1 ? "en" : "zh";
            UpdateUIText();
            ShowPage(1);
        }
        else if (_currentPage == 1)
            ShowPage(2);
        else if (_currentPage == 2)
        {
            _target = _pathBox.Text.Trim();
            if (string.IsNullOrEmpty(_target))
            {
                MessageBox.Show(T("请选择安装目录", "Please select installation directory"),
                    "EyeForge", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            ShowPage(3);
            BeginInvoke(new Action(RunInstall));
        }
        else if (_currentPage == 4)
        {
            if (_shortcutCheck.Checked)
                CreateShortcut(_target);
            Application.Exit();
        }
    }

    void RunInstall()
    {
        try
        {
            byte[] self = File.ReadAllBytes(
                System.Reflection.Assembly.GetExecutingAssembly().Location);
            int idx = FindMarker(self);
            if (idx < 0) throw new Exception("Invalid installer package");

            idx += Marker.Length;
            string zipPath = Path.Combine(Path.GetTempPath(), "ef_payload.zip");
            File.WriteAllBytes(zipPath, CopySlice(self, idx, self.Length - idx));

            _status.Text = T("解压文件...", "Extracting files...");

            using (ZipArchive archive = ZipFile.OpenRead(zipPath))
            {
                _progress.Style = ProgressBarStyle.Blocks;
                _progress.Maximum = archive.Entries.Count;
                _progress.Value = 0;

                foreach (ZipArchiveEntry entry in archive.Entries)
                {
                    string dest = Path.Combine(_target, entry.FullName);
                    string parent = Path.GetDirectoryName(dest);
                    if (!Directory.Exists(parent))
                        Directory.CreateDirectory(parent);
                    if (entry.Name != "")
                    {
                        if (File.Exists(dest)) File.Delete(dest);
                        entry.ExtractToFile(dest);
                    }
                    _progress.Value++;
                    Application.DoEvents();
                }
            }

            File.Delete(zipPath);

            if (_autoDepCheck.Checked)
            {
                _status.Text = T("安装依赖...", "Installing dependencies...");
                _progress.Style = ProgressBarStyle.Marquee;

                if (!CheckPythonInstalled())
                {
                    string msg = T(
                        "未检测到 Python！\n\n请先安装 Python 3.10+：\nhttps://www.python.org/downloads/\n\n安装时务必勾选 [Add Python to PATH]",
                        "Python not found!\n\nDownload Python 3.10+:\nhttps://www.python.org/downloads/\n\nMake sure to check [Add Python to PATH]");
                    throw new Exception(msg);
                }

                System.Diagnostics.ProcessStartInfo psi = new System.Diagnostics.ProcessStartInfo();
                psi.FileName = "cmd";
                psi.Arguments = "/c " + Path.Combine(_target, "install.bat");
                psi.WindowStyle = System.Diagnostics.ProcessWindowStyle.Hidden;
                psi.WorkingDirectory = _target;
                System.Diagnostics.Process p = System.Diagnostics.Process.Start(psi);
                p.WaitForExit();
            }

            BeginInvoke(new Action(() =>
            {
                ShowPage(4);
                _nextBtn.Enabled = true;
                _cancelBtn.Enabled = true;
            }));
        }
        catch (Exception ex)
        {
            string failed = T("安装失败", "Installation failed");
            _status.Text = failed + ":\n" + ex.Message;
            _progress.Style = ProgressBarStyle.Blocks;
            _progress.Maximum = 1;
            _progress.Value = 1;
            _nextBtn.Text = T("完成", "Finish");
            _nextBtn.Enabled = true;
            _prevBtn.Visible = false;
            _cancelBtn.Enabled = true;
        }
    }

    bool CheckPythonInstalled()
    {
        try
        {
            System.Diagnostics.ProcessStartInfo psi = new System.Diagnostics.ProcessStartInfo();
            psi.FileName = "cmd";
            psi.Arguments = "/c where python >nul 2>nul";
            psi.WindowStyle = System.Diagnostics.ProcessWindowStyle.Hidden;
            psi.CreateNoWindow = true;
            System.Diagnostics.Process p = System.Diagnostics.Process.Start(psi);
            p.WaitForExit(5000);
            return p.ExitCode == 0;
        }
        catch
        {
            return false;
        }
    }

    void CreateShortcut(string targetDir)
    {
        string desktop = Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory);
        string shortcutPath = Path.Combine(desktop, "EyeForge.lnk");
        string startExe = Path.Combine(targetDir, "start.exe");

        Type t = Type.GetTypeFromCLSID(new Guid("00021401-0000-0000-C000-000000000046"));
        dynamic shell = Activator.CreateInstance(t);
        try
        {
            dynamic shortcut = shell.CreateShortcut(shortcutPath);
            shortcut.TargetPath = startExe;
            shortcut.WorkingDirectory = targetDir;
            shortcut.Description = "EyeForge - AI Desktop Assistant";
            shortcut.Save();
        }
        finally
        {
            System.Runtime.InteropServices.Marshal.ReleaseComObject(shell);
        }
    }

    static int FindMarker(byte[] data)
    {
        for (int i = data.Length - Marker.Length; i >= 0; i--)
        {
            bool found = true;
            for (int j = 0; j < Marker.Length; j++)
                if (data[i + j] != Marker[j]) { found = false; break; }
            if (found) return i;
        }
        return -1;
    }

    static byte[] CopySlice(byte[] src, int offset, int length)
    {
        byte[] result = new byte[length];
        Buffer.BlockCopy(src, offset, result, 0, length);
        return result;
    }
}
