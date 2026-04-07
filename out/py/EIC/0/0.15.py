import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.ticker import MultipleLocator, AutoLocator
import tkinter as tk
from tkinter import filedialog, ttk, colorchooser
import os
import numpy as np

class ChromatogramApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Chromatogram Professional Viewer")
        
        # 1. Дефолтные настройки
        self.default_style = {
            'font_family': 'Arial',
            'title_size': 14.0,
            'label_size': 12.0,
            'tick_size': 10.0,
            'legend_size': 10.0,
            'line_width': 1.5,
            'grid_on': True,
            'grid_alpha': 0.3
        }
        self.style_params = self.default_style.copy()
        
        # 2. Инициализация данных
        self.plots_registry = []
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.global_y_max = 0
        
        self.grid_var = tk.BooleanVar(value=self.style_params['grid_on'])
        self.mode_var = tk.StringVar(value="Абсолютный")
        
        # 3. Интерфейс Matplotlib
        self.fig, self.ax = plt.subplots(figsize=(10, 6))
        self.canvas = FigureCanvasTkAgg(self.fig, master=self.root)
        self.canvas.get_tk_widget().pack(side=tk.TOP, fill=tk.BOTH, expand=1)
        
        self.create_control_window()
        
    def create_control_window(self):
        self.ctrl_win = tk.Toplevel(self.root)
        self.ctrl_win.title("Панель управления")
        self.ctrl_win.geometry("650x850")
        self.ctrl_win.attributes('-topmost', True)
        
        self.notebook = ttk.Notebook(self.ctrl_win)
        self.notebook.pack(fill="both", expand=True, padx=5, pady=5)
        
        self.tab_files = ttk.Frame(self.notebook)
        self.tab_scale = ttk.Frame(self.notebook)
        self.tab_style = ttk.Frame(self.notebook)
        
        self.notebook.add(self.tab_files, text=" 📂 Файлы ")
        self.notebook.add(self.tab_scale, text=" 📏 Масштаб ")
        self.notebook.add(self.tab_style, text=" 🎨 Оформление ")
        
        self.setup_tab_files()
        self.setup_tab_scale()
        self.setup_tab_style()
        
        btn_frame = ttk.Frame(self.ctrl_win, padding=10)
        btn_frame.pack(fill="x")
        ttk.Button(btn_frame, text="💾 Сохранить график (SVG)", command=self.save_svg).pack(fill="x")

    def setup_tab_files(self):
        frame_btns = ttk.Frame(self.tab_files, padding=10)
        frame_btns.pack(fill="x")
        ttk.Button(frame_btns, text="➕ Добавить файлы", command=self.add_files).pack(side="left", padx=5)
        ttk.Button(frame_btns, text="🗑️ Очистить всё", command=self.clear_all).pack(side="left", padx=5)
        
        self.files_container = ttk.Frame(self.tab_files)
        self.files_container.pack(fill="both", expand=True, padx=10, pady=5)
        
        self.canvas_files = tk.Canvas(self.files_container)
        self.scrollbar = ttk.Scrollbar(self.files_container, orient="vertical", command=self.canvas_files.yview)
        self.scrollable_frame = ttk.Frame(self.canvas_files)
        self.scrollable_frame.bind("<Configure>", lambda e: self.canvas_files.configure(scrollregion=self.canvas_files.bbox("all")))
        self.canvas_files.create_window((0, 0), window=self.scrollable_frame, anchor="nw")
        self.canvas_files.configure(yscrollcommand=self.scrollbar.set)
        self.canvas_files.pack(side="left", fill="both", expand=True)
        self.scrollbar.pack(side="right", fill="y")

    def setup_tab_scale(self):
        frame = ttk.Frame(self.tab_scale, padding=20)
        frame.pack(fill="both")
        
        # Режим
        ttk.Label(frame, text="Режим отображения:", font=('Arial', 10, 'bold')).pack(anchor="w", pady=5)
        ttk.Radiobutton(frame, text="Абсолютный (Counts)", variable=self.mode_var, value="Абсолютный", command=self.refresh_plots).pack(anchor="w")
        ttk.Radiobutton(frame, text="Нормированный (100% в окне)", variable=self.mode_var, value="Нормированный", command=self.refresh_plots).pack(anchor="w")
        
        ttk.Separator(frame, orient="horizontal").pack(fill="x", pady=15)
        
        # Диапазон X
        ttk.Label(frame, text="Диапазон времени (мин):", font=('Arial', 10, 'bold')).pack(anchor="w")
        for attr in ['min', 'max']:
            f = ttk.Frame(frame)
            f.pack(fill="x", pady=2)
            scale = ttk.Scale(f, from_=0, to=1, orient="horizontal", command=self.sync_scale_to_entry_time)
            scale.pack(side="left", fill="x", expand=True)
            entry = ttk.Entry(f, width=8)
            entry.pack(side="right", padx=5)
            entry.bind("<Return>", self.sync_entry_to_scale_time)
            setattr(self, f'scale_{attr}', scale)
            setattr(self, f'ent_{attr}', entry)

        ttk.Separator(frame, orient="horizontal").pack(fill="x", pady=15)

        # Шаг шкалы
        ttk.Label(frame, text="Шаг делений (0 = авто):", font=('Arial', 10, 'bold')).pack(anchor="w")
        f_ticks = ttk.Frame(frame)
        f_ticks.pack(fill="x", pady=5)
        
        ttk.Label(f_ticks, text="X:").pack(side="left")
        self.ent_x_step = ttk.Entry(f_ticks, width=8)
        self.ent_x_step.pack(side="left", padx=5)
        self.ent_x_step.insert(0, "0")
        self.ent_x_step.bind("<Return>", lambda e: self.refresh_plots())

        ttk.Label(f_ticks, text="Y:").pack(side="left", padx=(10, 0))
        self.ent_y_step = ttk.Entry(f_ticks, width=8)
        self.ent_y_step.pack(side="left", padx=5)
        self.ent_y_step.insert(0, "0")
        self.ent_y_step.bind("<Return>", lambda e: self.refresh_plots())
        
        ttk.Button(frame, text="🔍 Подогнать Y под окно (в Абс.)", command=self.manual_fit).pack(fill="x", pady=20)

    def setup_tab_style(self):
        frame = ttk.Frame(self.tab_style, padding=20)
        frame.pack(fill="both")
        frame.columnconfigure(1, weight=1)

        # Шрифт
        ttk.Label(frame, text="Шрифт:").grid(row=0, column=0, sticky="w")
        self.font_combo = ttk.Combobox(frame, values=["Arial", "Times New Roman", "Verdana", "Courier New", "Tahoma"])
        self.font_combo.set(self.style_params['font_family'])
        self.font_combo.grid(row=0, column=1, sticky="ew", pady=5, columnspan=2)
        self.font_combo.bind("<<ComboboxSelected>>", self.update_style)

        self.style_widgets = {}
        self.create_style_row(frame, "Размер заголовка", 'title_size', 8, 24, 1)
        self.create_style_row(frame, "Размер осей", 'label_size', 8, 20, 2)
        self.create_style_row(frame, "Размер чисел", 'tick_size', 6, 16, 3)
        self.create_style_row(frame, "Размер легенды", 'legend_size', 6, 16, 4)
        
        ttk.Separator(frame, orient="horizontal").grid(row=5, column=0, columnspan=3, sticky="ew", pady=10)
        
        self.create_style_row(frame, "Толщина линий", 'line_width', 0.5, 5.0, 6)
        
        ttk.Checkbutton(frame, text="Включить сетку", variable=self.grid_var, command=self.update_style).grid(row=7, column=0, columnspan=3, sticky="w")
        self.create_style_row(frame, "Прозрачность сетки", 'grid_alpha', 0.1, 1.0, 8)

        ttk.Button(frame, text="🔄 Сбросить оформление", command=self.reset_style_to_default).grid(row=9, column=0, columnspan=3, pady=20, sticky="ew")

    def create_style_row(self, master, label, param_name, f, t, row):
        ttk.Label(master, text=label + ":").grid(row=row, column=0, sticky="w")
        s = ttk.Scale(master, from_=f, to=t, orient="horizontal", command=lambda e, p=param_name: self.on_style_slider(p, e))
        s.set(self.style_params[param_name])
        s.grid(row=row, column=1, sticky="ew", pady=2, padx=5)
        e = ttk.Entry(master, width=5)
        e.insert(0, f"{self.style_params[param_name]:.1f}")
        e.grid(row=row, column=2, padx=5)
        e.bind("<Return>", lambda event, p=param_name: self.on_style_entry(p))
        self.style_widgets[param_name] = {'scale': s, 'entry': e}

    def reset_style_to_default(self):
        self.style_params = self.default_style.copy()
        self.grid_var.set(self.style_params['grid_on'])
        self.font_combo.set(self.style_params['font_family'])
        for p, widgets in self.style_widgets.items():
            widgets['scale'].set(self.style_params[p])
            widgets['entry'].delete(0, tk.END)
            widgets['entry'].insert(0, f"{self.style_params[p]:.1f}")
        self.update_style()

    def sync_scale_to_entry_time(self, event):
        for attr in ['min', 'max']:
            ent, scale = getattr(self, f'ent_{attr}'), getattr(self, f'scale_{attr}')
            ent.delete(0, tk.END)
            ent.insert(0, f"{scale.get():.3f}")
        self.refresh_plots()

    def sync_entry_to_scale_time(self, event):
        try:
            self.scale_min.set(float(self.ent_min.get()))
            self.scale_max.set(float(self.ent_max.get()))
            self.refresh_plots()
        except: pass

    def on_style_slider(self, param, val):
        self.style_params[param] = float(val)
        ent = self.style_widgets[param]['entry']
        ent.delete(0, tk.END)
        ent.insert(0, f"{float(val):.1f}")
        self.update_style()

    def on_style_entry(self, param):
        try:
            val = float(self.style_widgets[param]['entry'].get())
            self.style_widgets[param]['scale'].set(val)
            self.style_params[param] = val
            self.update_style()
        except: pass

    def update_style(self, event=None):
        if not hasattr(self, 'font_combo'): return
        self.style_params['font_family'] = self.font_combo.get()
        self.style_params['grid_on'] = self.grid_var.get()
        
        plt.rcParams['font.family'] = self.style_params['font_family']
        self.ax.set_title("EIC Chromatograms", fontsize=self.style_params['title_size'])
        self.ax.set_xlabel("Время (мин)", fontsize=self.style_params['label_size'])
        self.ax.tick_params(labelsize=self.style_params['tick_size'])
        
        for item in self.plots_registry:
            item['line'].set_linewidth(self.style_params['line_width'])
        
        self.ax.grid(False)
        if self.style_params['grid_on']:
            self.ax.grid(True, alpha=self.style_params['grid_alpha'])
        if self.ax.get_legend():
            plt.setp(self.ax.get_legend().get_texts(), fontsize=self.style_params['legend_size'])
        self.canvas.draw_idle()

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
                line, = self.ax.plot(t, y, label=label, lw=self.style_params['line_width'], color=color, linestyle='-')
                self.plots_registry.append({'t': t, 'y_orig': y, 'line': line, 'color': color, 'label': label, 'linestyle': '-'})
                self.all_data_bounds[0] = min(self.all_data_bounds[0], np.min(t))
                self.all_data_bounds[1] = max(self.all_data_bounds[1], np.max(t))
                self.global_y_max = max(self.global_y_max, np.max(y))

        self.update_ui_list()
        self.update_style()
        self.refresh_plots()

    def update_ui_list(self):
        for widget in self.scrollable_frame.winfo_children(): widget.destroy()
        line_styles = ["-", "--", ":", "-."]
        
        for i, item in enumerate(self.plots_registry):
            frame = ttk.Frame(self.scrollable_frame)
            frame.pack(fill="x", pady=2, padx=5)
            
            tk.Button(frame, bg=item['color'], width=2, relief="flat", command=lambda idx=i: self.pick_color(idx)).pack(side="left", padx=2)
            
            cb_style = ttk.Combobox(frame, values=line_styles, width=3, state="readonly")
            cb_style.set(item['linestyle'])
            cb_style.pack(side="left", padx=2)
            cb_style.bind("<<ComboboxSelected>>", lambda e, idx=i, cb=cb_style: self.on_style_change(idx, cb))
            
            name_var = tk.StringVar(value=item['label'])
            ent = ttk.Entry(frame, textvariable=name_var)
            ent.pack(side="left", fill="x", expand=True, padx=2)
            name_var.trace_add("write", lambda *args, idx=i, var=name_var: self.on_label_change(idx, var))

        self.scale_min.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        self.scale_max.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        if self.scale_min.get() == 0: self.scale_min.set(self.all_data_bounds[0])
        if self.scale_max.get() == 1: self.scale_max.set(self.all_data_bounds[1])
        self.sync_scale_to_entry_time(None)

    def on_style_change(self, index, cb):
        style = cb.get()
        self.plots_registry[index]['linestyle'] = style
        self.plots_registry[index]['line'].set_linestyle(style)
        self.canvas.draw_idle()

    def on_label_change(self, index, var):
        new_label = var.get()
        self.plots_registry[index]['label'] = new_label
        self.plots_registry[index]['line'].set_label(new_label)
        self.ax.legend(loc='upper right', fontsize=self.style_params['legend_size'])
        self.canvas.draw_idle()

    def pick_color(self, index):
        color = colorchooser.askcolor(initialcolor=self.plots_registry[index]['color'])[1]
        if color:
            self.plots_registry[index]['color'] = color
            self.plots_registry[index]['line'].set_color(color)
            self.update_ui_list()
            self.refresh_plots()

    def refresh_plots(self):
        if not self.plots_registry: return
        t_start, t_end = self.scale_min.get(), self.scale_max.get()
        if t_start >= t_end: t_end = t_start + 0.01
        self.ax.set_xlim(t_start, t_end)
        
        # Применение шага шкалы
        for axis, ent in zip([self.ax.xaxis, self.ax.yaxis], [self.ent_x_step, self.ent_y_step]):
            try:
                step = float(ent.get())
                axis.set_major_locator(MultipleLocator(step) if step > 0 else AutoLocator())
            except: axis.set_major_locator(AutoLocator())

        if self.mode_var.get() == "Нормированный":
            self.ax.set_ylabel('Относительная интенсивность (%)', fontsize=self.style_params['label_size'])
            for item in self.plots_registry:
                mask = (item['t'] >= t_start) & (item['t'] <= t_end)
                visible_y = item['y_orig'][mask]
                if len(visible_y) > 0 and np.max(visible_y) > 0:
                    item['line'].set_ydata((item['y_orig'] / np.max(visible_y)) * 100)
                else: item['line'].set_ydata(item['y_orig'] * 0)
            self.ax.set_ylim(-2, 105)
        else:
            self.ax.set_ylabel('Интенсивность (Counts)', fontsize=self.style_params['label_size'])
            for item in self.plots_registry: item['line'].set_ydata(item['y_orig'])
            self.ax.set_ylim(-self.global_y_max * 0.02, self.global_y_max * 1.05)
        
        self.ax.legend(loc='upper right', fontsize=self.style_params['legend_size'])
        self.canvas.draw_idle()

    def manual_fit(self):
        if self.mode_var.get() == "Абсолютный":
            t_start, t_end = self.scale_min.get(), self.scale_max.get()
            local_max = 0
            for item in self.plots_registry:
                mask = (item['t'] >= t_start) & (item['t'] <= t_end)
                if any(mask): local_max = max(local_max, np.max(item['y_orig'][mask]))
            if local_max > 0: self.ax.set_ylim(-local_max * 0.02, local_max * 1.05)
            self.canvas.draw()

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

    def clear_all(self):
        for item in self.plots_registry: item['line'].remove()
        self.plots_registry.clear()
        self.global_y_max = 0
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.ax.legend_ = None
        self.ax.set_xlim(0, 1); self.ax.set_ylim(0, 1)
        self.update_ui_list()
        self.canvas.draw()

    def save_svg(self):
        save_path = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        if save_path: self.fig.savefig(save_path, format='svg', bbox_inches='tight')

if __name__ == "__main__":
    root = tk.Tk()
    app = ChromatogramApp(root)
    root.mainloop()