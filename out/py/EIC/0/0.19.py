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
        self.ui_ready = False
        
        # 1. Дефолтные настройки
        self.default_style = {
            'font_family': 'Arial',
            'title_size': 14.0,
            'label_size': 12.0,
            'tick_size': 10.0,
            'legend_size': 10.0,
            'line_width': 1.5,
            'grid_on': True,
            'grid_alpha': 0.3,
            'grid_width': 0.8,
            'spine_width': 1.0,
            'tick_width': 1.0,
            'x_tick_step': 0.0,     # 0 = Авто
            'y_tick_step': 0.0,     # 0 = Авто
            'title_text': 'EIC Chromatograms',
            'x_label': 'Время (мин)',
            'y_label_abs': 'Интенсивность (Counts)',
            'y_label_norm': 'Относительная интенсивность (%)',
            'show_legend': True
        }
        self.style_params = self.default_style.copy()
        
        # 2. Данные и переменные
        self.plots_registry = []
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.global_y_max = 0
        self.style_widgets = {}
        self.label_entries = {}
        
        self.grid_var = tk.BooleanVar(value=self.style_params['grid_on'])
        self.legend_var = tk.BooleanVar(value=self.style_params['show_legend'])
        self.mode_var = tk.StringVar(value="Абсолютный")
        
        # 3. Matplotlib интерфейс
        self.fig, self.ax = plt.subplots(figsize=(10, 6))
        self.canvas = FigureCanvasTkAgg(self.fig, master=self.root)
        self.canvas.get_tk_widget().pack(side=tk.TOP, fill=tk.BOTH, expand=1)
        
        # 4. Создание окон управления
        self.create_control_window()
        
        # Протокол закрытия (чтобы консоль не зависала)
        self.root.protocol("WM_DELETE_WINDOW", self.on_close)
        self.ui_ready = True

    def create_control_window(self):
        self.ctrl_win = tk.Toplevel(self.root)
        self.ctrl_win.title("Панель управления")
        self.ctrl_win.geometry("700x900")
        self.ctrl_win.protocol("WM_DELETE_WINDOW", self.on_close)
        
        self.notebook = ttk.Notebook(self.ctrl_win)
        self.notebook.pack(fill="both", expand=True, padx=5, pady=5)
        
        # Создание вкладок
        self.tab_files = ttk.Frame(self.notebook)
        self.tab_scale = ttk.Frame(self.notebook)
        self.tab_labels = ttk.Frame(self.notebook)
        self.tab_style = ttk.Frame(self.notebook)
        
        self.notebook.add(self.tab_files, text=" 📂 Файлы ")
        self.notebook.add(self.tab_scale, text=" 📏 Масштаб ")
        self.notebook.add(self.tab_labels, text=" 📝 Подписи ")
        self.notebook.add(self.tab_style, text=" 🎨 Оформление ")
        
        # Инициализация содержимого вкладок
        self.setup_tab_files()
        self.setup_tab_scale()
        self.setup_tab_labels()
        self.setup_tab_style()
        
        ttk.Button(self.ctrl_win, text="💾 Сохранить график (SVG)", command=self.save_svg).pack(fill="x", padx=15, pady=10)

    def setup_tab_files(self):
        frame_btns = ttk.Frame(self.tab_files, padding=10)
        frame_btns.pack(fill="x")
        ttk.Button(frame_btns, text="➕ Добавить файлы", command=self.add_files).pack(side="left", padx=5)
        ttk.Button(frame_btns, text="🗑️ Очистить всё", command=self.clear_all).pack(side="left", padx=5)
        
        lbl_hint = ttk.Label(self.tab_files, text="Цвет | Стиль | Толщина | Имя в легенде", font=("Arial", 8, "italic"))
        lbl_hint.pack(anchor="w", padx=15)

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
        
        ttk.Label(frame, text="Режим отображения:", font=('Arial', 10, 'bold')).pack(anchor="w", pady=5)
        ttk.Radiobutton(frame, text="Абсолютный (Counts)", variable=self.mode_var, value="Абсолютный", command=self.refresh_plots).pack(anchor="w")
        ttk.Radiobutton(frame, text="Нормированный (100% в окне)", variable=self.mode_var, value="Нормированный", command=self.refresh_plots).pack(anchor="w")
        
        ttk.Separator(frame, orient="horizontal").pack(fill="x", pady=15)
        
        ttk.Label(frame, text="Диапазон времени (мин):", font=('Arial', 10, 'bold')).pack(anchor="w")
        for attr in ['min', 'max']:
            f = ttk.Frame(frame); f.pack(fill="x", pady=2)
            scale = ttk.Scale(f, from_=0, to=1, orient="horizontal", command=self.sync_scale_to_entry_time)
            scale.pack(side="left", fill="x", expand=True)
            entry = ttk.Entry(f, width=10); entry.pack(side="right", padx=5)
            entry.bind("<Return>", self.sync_entry_to_scale_time)
            setattr(self, f'scale_{attr}', scale); setattr(self, f'ent_{attr}', entry)

        ttk.Separator(frame, orient="horizontal").pack(fill="x", pady=15)

        ttk.Label(frame, text="Шаг делений (0 = Авто):", font=('Arial', 10, 'bold')).pack(anchor="w")
        
        # Шаг X
        f_x = ttk.Frame(frame); f_x.pack(fill="x", pady=2)
        ttk.Label(f_x, text="X:", width=3).pack(side="left")
        self.scale_tick_x = ttk.Scale(f_x, from_=0, to=10, orient="horizontal", command=lambda e: self.on_tick_change('x'))
        self.scale_tick_x.pack(side="left", fill="x", expand=True)
        self.ent_tick_x = ttk.Entry(f_x, width=10); self.ent_tick_x.pack(side="right", padx=5)
        self.ent_tick_x.bind("<Return>", lambda e: self.on_tick_entry('x'))
        
        # Шаг Y
        f_y = ttk.Frame(frame); f_y.pack(fill="x", pady=2)
        ttk.Label(f_y, text="Y:", width=3).pack(side="left")
        self.scale_tick_y = ttk.Scale(f_y, from_=0, to=1000000, orient="horizontal", command=lambda e: self.on_tick_change('y'))
        self.scale_tick_y.pack(side="left", fill="x", expand=True)
        self.ent_tick_y = ttk.Entry(f_y, width=10); self.ent_tick_y.pack(side="right", padx=5)
        self.ent_tick_y.bind("<Return>", lambda e: self.on_tick_entry('y'))

        ttk.Button(frame, text="🔄 Сбросить шаг (Авто)", command=self.reset_ticks).pack(fill="x", pady=10)
        ttk.Button(frame, text="🔍 Подогнать Y под окно (в Абс.)", command=self.manual_fit).pack(fill="x", pady=10)

    def setup_tab_labels(self):
        frame = ttk.Frame(self.tab_labels, padding=20)
        frame.pack(fill="both")
        labels_config = [("Заголовок:", 'title_text'), ("Ось X:", 'x_label'), ("Ось Y (Абс):", 'y_label_abs'), ("Ось Y (Норм):", 'y_label_norm')]
        for txt, key in labels_config:
            ttk.Label(frame, text=txt).pack(anchor="w", pady=(5, 0))
            ent = ttk.Entry(frame); ent.insert(0, self.style_params[key]); ent.pack(fill="x", pady=2)
            ent.bind("<KeyRelease>", lambda e, k=key, en=ent: self.on_label_text_update(k, en))
            self.label_entries[key] = ent
        ttk.Separator(frame, orient="horizontal").pack(fill="x", pady=15)
        ttk.Checkbutton(frame, text="Показывать легенду", variable=self.legend_var, command=self.update_style).pack(anchor="w")

    def setup_tab_style(self):
        frame = ttk.Frame(self.tab_style, padding=20)
        frame.pack(fill="both")
        frame.columnconfigure(1, weight=1)

        ttk.Label(frame, text="Шрифт:").grid(row=0, column=0, sticky="w")
        self.font_combo = ttk.Combobox(frame, values=["Arial", "Times New Roman", "Verdana", "Courier New", "Tahoma"])
        self.font_combo.set(self.style_params['font_family'])
        self.font_combo.grid(row=0, column=1, sticky="ew", pady=5, columnspan=2)
        self.font_combo.bind("<<ComboboxSelected>>", self.update_style)

        self.create_style_row(frame, "Размер заголовка", 'title_size', 8, 24, 1)
        self.create_style_row(frame, "Размер осей", 'label_size', 8, 20, 2)
        self.create_style_row(frame, "Размер чисел", 'tick_size', 6, 16, 3)
        self.create_style_row(frame, "Размер легенды", 'legend_size', 6, 16, 4)
        ttk.Separator(frame, orient="horizontal").grid(row=5, column=0, columnspan=3, sticky="ew", pady=10)
        
        self.create_style_row(frame, "Толщина линий (общая)", 'line_width', 0.5, 5.0, 6)
        self.create_style_row(frame, "Толщина осей", 'spine_width', 0.1, 4.0, 7)
        self.create_style_row(frame, "Толщина делений", 'tick_width', 0.1, 4.0, 8)
        
        ttk.Separator(frame, orient="horizontal").grid(row=9, column=0, columnspan=3, sticky="ew", pady=10)
        ttk.Checkbutton(frame, text="Включить сетку", variable=self.grid_var, command=self.update_style).grid(row=10, column=0, columnspan=3, sticky="w")
        self.create_style_row(frame, "Толщина сетки", 'grid_width', 0.1, 3.0, 11)
        self.create_style_row(frame, "Прозрачность сетки", 'grid_alpha', 0.1, 1.0, 12)
        
        ttk.Button(frame, text="🔄 Сбросить всё", command=self.reset_all_settings).grid(row=14, column=0, columnspan=3, pady=20, sticky="ew")

    def create_style_row(self, master, label, param_name, f, t, row):
        ttk.Label(master, text=label + ":").grid(row=row, column=0, sticky="w")
        s = ttk.Scale(master, from_=f, to=t, orient="horizontal", command=lambda e, p=param_name: self.on_style_slider(p, e))
        s.set(self.style_params[param_name])
        s.grid(row=row, column=1, sticky="ew", pady=2, padx=5)
        e = ttk.Entry(master, width=6)
        e.insert(0, f"{self.style_params[param_name]:.1f}")
        e.grid(row=row, column=2, padx=5)
        e.bind("<Return>", lambda event, p=param_name: self.on_style_entry(p))
        self.style_widgets[param_name] = {'scale': s, 'entry': e}

    def on_tick_change(self, axis):
        if not self.ui_ready: return
        scale = self.scale_tick_x if axis == 'x' else self.scale_tick_y
        entry = self.ent_tick_x if axis == 'x' else self.ent_tick_y
        val = scale.get()
        self.style_params[f'{axis}_tick_step'] = val
        entry.delete(0, tk.END); entry.insert(0, f"{val:.2f}")
        self.refresh_plots()

    def on_tick_entry(self, axis):
        try:
            entry = self.ent_tick_x if axis == 'x' else self.ent_tick_y
            scale = self.scale_tick_x if axis == 'x' else self.scale_tick_y
            val = float(entry.get())
            scale.set(val)
            self.style_params[f'{axis}_tick_step'] = val
            self.refresh_plots()
        except: pass

    def reset_ticks(self):
        self.style_params['x_tick_step'] = 0.0
        self.style_params['y_tick_step'] = 0.0
        self.scale_tick_x.set(0); self.ent_tick_x.delete(0, tk.END); self.ent_tick_x.insert(0, "0.00")
        self.scale_tick_y.set(0); self.ent_tick_y.delete(0, tk.END); self.ent_tick_y.insert(0, "0.00")
        self.refresh_plots()

    def on_style_slider(self, param, val):
        if not self.ui_ready: return
        self.style_params[param] = float(val)
        ent = self.style_widgets[param]['entry']
        ent.delete(0, tk.END); ent.insert(0, f"{float(val):.1f}")
        if param == 'line_width':
            for item in self.plots_registry: item['line_width'] = float(val)
        self.update_style()

    def on_style_entry(self, param):
        try:
            val = float(self.style_widgets[param]['entry'].get())
            self.style_widgets[param]['scale'].set(val)
            self.style_params[param] = val
            if param == 'line_width':
                for item in self.plots_registry: item['line_width'] = val
            self.update_style()
        except: pass

    def update_style(self, event=None):
        if not self.ui_ready: return
        self.style_params['font_family'] = self.font_combo.get()
        plt.rcParams['font.family'] = self.style_params['font_family']
        self.ax.set_title(self.style_params['title_text'], fontsize=self.style_params['title_size'])
        self.ax.set_xlabel(self.style_params['x_label'], fontsize=self.style_params['label_size'])
        self.ax.tick_params(labelsize=self.style_params['tick_size'], width=self.style_params['tick_width'])
        for spine in self.ax.spines.values(): spine.set_linewidth(self.style_params['spine_width'])
        for item in self.plots_registry: item['line'].set_linewidth(item['line_width'])
        self.ax.grid(False)
        if self.grid_var.get():
            self.ax.grid(True, alpha=self.style_params['grid_alpha'], linewidth=self.style_params['grid_width'])
        if self.ax.get_legend(): self.ax.get_legend().remove()
        if self.legend_var.get() and self.plots_registry:
            self.ax.legend(loc='upper right', fontsize=self.style_params['legend_size'])
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
                line, = self.ax.plot(t, y, label=label, lw=self.style_params['line_width'], color=color)
                self.plots_registry.append({
                    't': t, 'y_orig': y, 'line': line, 'color': color, 
                    'label': label, 'linestyle': '-', 'line_width': self.style_params['line_width']
                })
                self.all_data_bounds[0] = min(self.all_data_bounds[0], np.min(t))
                self.all_data_bounds[1] = max(self.all_data_bounds[1], np.max(t))
                self.global_y_max = max(self.global_y_max, np.max(y))
        
        self.scale_tick_y.config(to=self.global_y_max / 2)
        self.update_ui_list(); self.update_style(); self.refresh_plots()

    def update_ui_list(self):
        for widget in self.scrollable_frame.winfo_children(): widget.destroy()
        for i, item in enumerate(self.plots_registry):
            frame = ttk.Frame(self.scrollable_frame); frame.pack(fill="x", pady=2, padx=5)
            tk.Button(frame, bg=item['color'], width=2, relief="flat", command=lambda idx=i: self.pick_color(idx)).pack(side="left", padx=2)
            cb = ttk.Combobox(frame, values=["-", "--", ":", "-."], width=3, state="readonly")
            cb.set(item['linestyle']); cb.pack(side="left", padx=2)
            cb.bind("<<ComboboxSelected>>", lambda e, idx=i, c=cb: self.on_line_style_change(idx, c))
            sp = tk.Spinbox(frame, from_=0.1, to=10.0, increment=0.1, width=4, command=lambda idx=i: self.on_indiv_width_change(idx))
            sp.delete(0, "end"); sp.insert(0, f"{item['line_width']:.1f}"); sp.pack(side="left", padx=2)
            item['width_spin'] = sp
            var = tk.StringVar(value=item['label'])
            ent = ttk.Entry(frame, textvariable=var); ent.pack(side="left", fill="x", expand=True, padx=2)
            var.trace_add("write", lambda *a, idx=i, v=var: self.on_label_change(idx, v))
        self.scale_min.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        self.scale_max.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        if self.scale_min.get() == 0: self.scale_min.set(self.all_data_bounds[0])
        if self.scale_max.get() == 1: self.scale_max.set(self.all_data_bounds[1])
        self.sync_scale_to_entry_time(None)

    def on_indiv_width_change(self, idx):
        try:
            self.plots_registry[idx]['line_width'] = float(self.plots_registry[idx]['width_spin'].get())
            self.update_style()
        except: pass

    def on_line_style_change(self, idx, cb):
        self.plots_registry[idx]['linestyle'] = cb.get()
        self.plots_registry[idx]['line'].set_linestyle(cb.get())
        self.canvas.draw_idle()

    def on_label_change(self, idx, var):
        self.plots_registry[idx]['label'] = var.get()
        self.plots_registry[idx]['line'].set_label(var.get())
        self.update_style()

    def on_label_text_update(self, key, entry):
        self.style_params[key] = entry.get()
        self.refresh_plots()

    def sync_scale_to_entry_time(self, event):
        if not self.ui_ready: return
        for attr in ['min', 'max']:
            ent, scale = getattr(self, f'ent_{attr}'), getattr(self, f'scale_{attr}')
            ent.delete(0, tk.END); ent.insert(0, f"{scale.get():.3f}")
        self.refresh_plots()

    def sync_entry_to_scale_time(self, event):
        try:
            self.scale_min.set(float(self.ent_min.get()))
            self.scale_max.set(float(self.ent_max.get()))
            self.refresh_plots()
        except: pass

    def refresh_plots(self):
        if not self.plots_registry or not self.ui_ready: return
        t_s, t_e = self.scale_min.get(), self.scale_max.get()
        if t_s >= t_e: t_e = t_s + 0.01
        self.ax.set_xlim(t_s, t_e)
        
        step_x = self.style_params['x_tick_step']
        self.ax.xaxis.set_major_locator(MultipleLocator(step_x) if step_x > 0 else AutoLocator())
        step_y = self.style_params['y_tick_step']
        self.ax.yaxis.set_major_locator(MultipleLocator(step_y) if step_y > 0 else AutoLocator())

        if self.mode_var.get() == "Нормированный":
            self.ax.set_ylabel(self.style_params['y_label_norm'], fontsize=self.style_params['label_size'])
            for item in self.plots_registry:
                mask = (item['t'] >= t_s) & (item['t'] <= t_e)
                v_y = item['y_orig'][mask]
                if len(v_y) > 0 and np.max(v_y) > 0: item['line'].set_ydata((item['y_orig'] / np.max(v_y)) * 100)
                else: item['line'].set_ydata(item['y_orig'] * 0)
            self.ax.set_ylim(-2, 105)
        else:
            self.ax.set_ylabel(self.style_params['y_label_abs'], fontsize=self.style_params['label_size'])
            for item in self.plots_registry: item['line'].set_ydata(item['y_orig'])
            self.ax.set_ylim(-self.global_y_max * 0.02, self.global_y_max * 1.05)
        self.update_style()

    def manual_fit(self):
        if self.mode_var.get() == "Абсолютный":
            t_s, t_e = self.scale_min.get(), self.scale_max.get()
            l_max = 0
            for item in self.plots_registry:
                mask = (item['t'] >= t_s) & (item['t'] <= t_e)
                if any(mask): l_max = max(l_max, np.max(item['y_orig'][mask]))
            if l_max > 0: self.ax.set_ylim(-l_max * 0.02, l_max * 1.05); self.canvas.draw()

    def pick_color(self, idx):
        color = colorchooser.askcolor(initialcolor=self.plots_registry[idx]['color'])[1]
        if color:
            self.plots_registry[idx]['color'] = color
            self.plots_registry[idx]['line'].set_color(color)
            self.update_ui_list(); self.refresh_plots()

    def reset_all_settings(self):
        self.style_params = self.default_style.copy()
        self.grid_var.set(self.style_params['grid_on'])
        self.legend_var.set(self.style_params['show_legend'])
        self.font_combo.set(self.style_params['font_family'])
        for p, w in self.style_widgets.items():
            w['scale'].set(self.style_params[p])
            w['entry'].delete(0, tk.END); w['entry'].insert(0, f"{self.style_params[p]:.1f}")
        for k, en in self.label_entries.items():
            en.delete(0, tk.END); en.insert(0, self.style_params[k])
        for item in self.plots_registry: item['line_width'] = self.style_params['line_width']
        self.reset_ticks()
        self.update_ui_list(); self.update_style()

    def load_data(self, path):
        try:
            df = pd.read_csv(path, sep=';', decimal=',', comment='#', header=None)
            return pd.to_numeric(df.iloc[:, 1], errors='coerce').values, pd.to_numeric(df.iloc[:, 2], errors='coerce').values
        except: return None, None

    def clear_all(self):
        for item in self.plots_registry: item['line'].remove()
        self.plots_registry.clear(); self.global_y_max = 0; self.all_data_bounds = [float('inf'), float('-inf')]
        self.ax.set_xlim(0, 1); self.ax.set_ylim(0, 1); self.update_ui_list(); self.canvas.draw()

    def save_svg(self):
        path = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        if path: self.fig.savefig(path, format='svg', bbox_inches='tight')

    def on_close(self):
        plt.close('all'); self.root.quit(); self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk()
    app = ChromatogramApp(root)
    root.mainloop()