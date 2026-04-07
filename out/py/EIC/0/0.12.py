import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
import tkinter as tk
from tkinter import filedialog, ttk, colorchooser
import os
import numpy as np

class ChromatogramApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Chromatogram Viewer")
        
        # Данные
        self.plots_registry = []
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.global_y_max = 0
        
        # Основной интерфейс (График)
        self.fig, self.ax = plt.subplots(figsize=(10, 6))
        self.canvas = FigureCanvasTkAgg(self.fig, master=self.root)
        self.canvas.get_tk_widget().pack(side=tk.TOP, fill=tk.BOTH, expand=1)
        
        # Создание всплывающего окна настроек
        self.create_control_window()
        
    def create_control_window(self):
        self.ctrl_win = tk.Toplevel(self.root)
        self.ctrl_win.title("Настройки и Файлы")
        self.ctrl_win.geometry("450x600")
        self.ctrl_win.attributes('-topmost', True)
        
        # --- Секция управления файлами ---
        frame_top = ttk.Frame(self.ctrl_win, padding=10)
        frame_top.pack(fill="x")
        ttk.Button(frame_top, text="➕ Добавить файлы", command=self.add_files).pack(side="left", padx=5)
        ttk.Button(frame_top, text="🗑️ Очистить всё", command=self.clear_all).pack(side="left", padx=5)
        
        # --- Секция списка файлов с выбором цвета ---
        lbl_files = ttk.Label(self.ctrl_win, text="Список файлов (нажмите на цвет для изменения):", font=('Arial', 10, 'bold'))
        lbl_files.pack(pady=(10, 0))
        
        self.files_container = ttk.Frame(self.ctrl_win)
        self.files_container.pack(fill="both", expand=True, padx=10, pady=5)
        
        self.canvas_files = tk.Canvas(self.files_container, height=150)
        self.scrollbar = ttk.Scrollbar(self.files_container, orient="vertical", command=self.canvas_files.yview)
        self.scrollable_frame = ttk.Frame(self.canvas_files)

        self.scrollable_frame.bind("<Configure>", lambda e: self.canvas_files.configure(scrollregion=self.canvas_files.bbox("all")))
        self.canvas_files.create_window((0, 0), window=self.scrollable_frame, anchor="nw")
        self.canvas_files.configure(yscrollcommand=self.scrollbar.set)

        self.canvas_files.pack(side="left", fill="both", expand=True)
        self.scrollbar.pack(side="right", fill="y")

        # --- Секция режима ---
        frame_mode = ttk.LabelFrame(self.ctrl_win, text="Режим масштабирования", padding=10)
        frame_mode.pack(fill="x", padx=10, pady=5)
        
        self.mode_var = tk.StringVar(value="Абсолютный")
        ttk.Radiobutton(frame_mode, text="Абсолютный (Counts)", variable=self.mode_var, 
                        value="Абсолютный", command=self.refresh_plots).pack(anchor="w")
        ttk.Radiobutton(frame_mode, text="Нормированный (100% в окне)", variable=self.mode_var, 
                        value="Нормированный", command=self.refresh_plots).pack(anchor="w")
        
        # --- Секция диапазона времени ---
        frame_time = ttk.LabelFrame(self.ctrl_win, text="Диапазон времени (мин)", padding=10)
        frame_time.pack(fill="x", padx=10, pady=5)
        
        self.scale_min = ttk.Scale(frame_time, from_=0, to=1, orient="horizontal", command=self.on_slider_move)
        self.scale_min.pack(fill="x")
        self.scale_max = ttk.Scale(frame_time, from_=0, to=1, orient="horizontal", command=self.on_slider_move)
        self.scale_max.pack(fill="x")
        
        self.lbl_range = ttk.Label(frame_time, text="Диапазон: ---")
        self.lbl_range.pack()

        # --- Кнопки действий ---
        frame_actions = ttk.Frame(self.ctrl_win, padding=10)
        frame_actions.pack(fill="x")
        ttk.Button(frame_actions, text="🔍 Подогнать Y (Абс.)", command=self.manual_fit).pack(side="left", fill="x", expand=True, padx=2)
        ttk.Button(frame_actions, text="💾 Сохранить SVG", command=self.save_svg).pack(side="left", fill="x", expand=True, padx=2)

    def load_data(self, file_path):
        try:
            df = pd.read_csv(file_path, sep=';', decimal=',', comment='#', header=None)
            if df.shape[1] >= 3:
                time = pd.to_numeric(df.iloc[:, 1], errors='coerce')
                intensity = pd.to_numeric(df.iloc[:, 2], errors='coerce')
                mask = time.notna() & intensity.notna()
                return time[mask].values, intensity[mask].values
            return None, None
        except Exception as e:
            print(f"Ошибка чтения: {e}")
            return None, None

    def add_files(self):
        file_paths = filedialog.askopenfilenames(filetypes=[("Data", "*.csv *.txt"), ("All", "*.*")])
        if not file_paths: return

        colors = plt.rcParams['axes.prop_cycle'].by_key()['color']

        for path in file_paths:
            with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                header = f.readline()
                label = header.split("Scan")[0].replace('#"', '').strip() if "EIC" in header else os.path.basename(path)
            
            t, y = self.load_data(path)
            if t is not None and len(t) > 0:
                color = colors[len(self.plots_registry) % len(colors)]
                line, = self.ax.plot(t, y, label=label, lw=1.2, color=color)
                
                self.plots_registry.append({
                    't': t, 'y_orig': y, 'line': line, 
                    'color': color, 'label': label
                })
                
                # Обновление глобальных границ времени
                self.all_data_bounds[0] = min(self.all_data_bounds[0], np.min(t))
                self.all_data_bounds[1] = max(self.all_data_bounds[1], np.max(t))
                self.global_y_max = max(self.global_y_max, np.max(y))

        self.update_ui_elements()
        self.refresh_plots()

    def update_ui_elements(self):
        """Обновляет слайдеры и список файлов в интерфейсе"""
        # Слайдеры
        self.scale_min.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        self.scale_max.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        
        # Если это первая загрузка, ставим ползунки по краям
        if self.scale_min.get() == 0 and self.scale_max.get() == 1:
            self.scale_min.set(self.all_data_bounds[0])
            self.scale_max.set(self.all_data_bounds[1])

        # Список файлов
        for widget in self.scrollable_frame.winfo_children():
            widget.destroy()

        for i, item in enumerate(self.plots_registry):
            frame = ttk.Frame(self.scrollable_frame)
            frame.pack(fill="x", pady=2)
            
            # Кнопка-индикатор цвета
            btn_col = tk.Button(frame, bg=item['color'], width=2, relief="flat",
                                command=lambda idx=i: self.pick_color(idx))
            btn_col.pack(side="left", padx=5)
            
            ttk.Label(frame, text=item['label'], font=('Arial', 9)).pack(side="left")

    def pick_color(self, index):
        color = colorchooser.askcolor(initialcolor=self.plots_registry[index]['color'])[1]
        if color:
            self.plots_registry[index]['color'] = color
            self.plots_registry[index]['line'].set_color(color)
            self.update_ui_elements()
            self.refresh_plots()

    def on_slider_move(self, event):
        self.refresh_plots()

    def refresh_plots(self):
        if not self.plots_registry: return
        
        t_start = self.scale_min.get()
        t_end = self.scale_max.get()
        if t_start >= t_end: t_end = t_start + 0.01
        
        self.lbl_range.config(text=f"Диапазон: {t_start:.2f} - {t_end:.2f} мин")
        self.ax.set_xlim(t_start, t_end)
        
        mode = self.mode_var.get()
        
        if mode == "Нормированный":
            self.ax.set_ylabel('Относительная интенсивность (%)')
            for item in self.plots_registry:
                mask = (item['t'] >= t_start) & (item['t'] <= t_end)
                visible_y = item['y_orig'][mask]
                if len(visible_y) > 0 and np.max(visible_y) > 0:
                    local_max = np.max(visible_y)
                    item['line'].set_ydata((item['y_orig'] / local_max) * 100)
                else:
                    item['line'].set_ydata(item['y_orig'] * 0)
            self.ax.set_ylim(-2, 105)
        else:
            self.ax.set_ylabel('Интенсивность (Counts)')
            for item in self.plots_registry:
                item['line'].set_ydata(item['y_orig'])
            self.ax.set_ylim(-self.global_y_max * 0.02, self.global_y_max * 1.05)

        self.ax.legend(loc='upper right', fontsize='x-small')
        self.canvas.draw()

    def manual_fit(self):
        if self.mode_var.get() == "Абсолютный":
            t_start, t_end = self.scale_min.get(), self.scale_max.get()
            local_max = 0
            for item in self.plots_registry:
                mask = (item['t'] >= t_start) & (item['t'] <= t_end)
                if any(mask):
                    local_max = max(local_max, np.max(item['y_orig'][mask]))
            if local_max > 0:
                self.ax.set_ylim(-local_max * 0.02, local_max * 1.05)
                self.canvas.draw()

    def clear_all(self):
        for item in self.plots_registry: item['line'].remove()
        self.plots_registry.clear()
        self.global_y_max = 0
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.ax.legend_ = None
        self.ax.set_xlim(0, 1)
        self.ax.set_ylim(0, 1)
        self.update_ui_elements()
        self.canvas.draw()

    def save_svg(self):
        save_path = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        if save_path:
            self.fig.savefig(save_path, format='svg', bbox_inches='tight')

if __name__ == "__main__":
    root = tk.Tk()
    app = ChromatogramApp(root)
    root.mainloop()