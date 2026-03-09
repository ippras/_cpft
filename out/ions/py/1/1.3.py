import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.ticker import MultipleLocator, AutoLocator, NullLocator
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
            'font_family': 'Arial', 'title_size': 14.0, 'label_size': 12.0, 'tick_size': 10.0,
            'legend_size': 10.0, 'line_width': 1.5, 'grid_on': True, 'grid_alpha': 0.3,
            'grid_width': 0.8, 'spine_width': 1.0, 'tick_width': 1.0,
            'x_major_step': 0.0, 'y_major_step': 0.0, 'x_minor_step': 0.0, 'y_minor_step': 0.0,
            'title_text': 'EIC Chromatograms', 'x_label': 'Время (мин)',
            'y_label_abs': 'Интенсивность (Counts)', 'y_label_norm': 'Относительная интенсивность (%)',
            'show_legend': True
        }
        self.style_params = self.default_style.copy()
        
        # 2. Данные
        self.plots_registry = []
        self.helper_lines = []    # {type, pos, start, end, color, width, text, label_pos, layer, font_size}
        self.helper_artists = []
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.global_y_max = 0
        self.style_widgets = {}
        self.label_entries = {}
        
        self.grid_var = tk.BooleanVar(value=self.style_params['grid_on'])
        self.legend_var = tk.BooleanVar(value=self.style_params['show_legend'])
        self.mode_var = tk.StringVar(value="Абсолютный")
        
        # 3. Matplotlib
        self.fig, self.ax = plt.subplots(figsize=(10, 6))
        self.canvas = FigureCanvasTkAgg(self.fig, master=self.root)
        self.canvas.get_tk_widget().pack(side=tk.TOP, fill=tk.BOTH, expand=1)
        
        # 4. Интерфейс
        self.create_control_window()
        self.root.protocol("WM_DELETE_WINDOW", self.on_close)
        self.ui_ready = True

    def create_control_window(self):
        self.ctrl_win = tk.Toplevel(self.root)
        self.ctrl_win.title("Панель управления")
        self.ctrl_win.geometry("1150x850")
        self.ctrl_win.protocol("WM_DELETE_WINDOW", self.on_close)
        
        self.notebook = ttk.Notebook(self.ctrl_win)
        self.notebook.pack(fill="both", expand=True, padx=5, pady=5)
        
        self.tab_files = ttk.Frame(self.notebook)
        self.tab_scale = ttk.Frame(self.notebook)
        self.tab_labels = ttk.Frame(self.notebook)
        self.tab_helpers = ttk.Frame(self.notebook)
        self.tab_style = ttk.Frame(self.notebook)
        
        self.notebook.add(self.tab_files, text=" 📂 Файлы ")
        self.notebook.add(self.tab_scale, text=" 📏 Масштаб ")
        self.notebook.add(self.tab_labels, text=" 📝 Подписи ")
        self.notebook.add(self.tab_helpers, text=" 📍 Линии ")
        self.notebook.add(self.tab_style, text=" 🎨 Оформление ")
        
        self.setup_tab_files()
        self.setup_tab_scale()
        self.setup_tab_labels()
        self.setup_tab_helpers()
        self.setup_tab_style()
        
        ttk.Button(self.ctrl_win, text="💾 Сохранить график (SVG)", command=self.save_svg).pack(fill="x", padx=15, pady=10)

    # --- ВКЛАДКА ЛИНИЙ ---
    def setup_tab_helpers(self):
        frame = ttk.Frame(self.tab_helpers, padding=10); frame.pack(fill="both", expand=True)
        
        f_add = ttk.LabelFrame(frame, text="Добавить линию", padding=10); f_add.pack(fill="x")
        self.h_type = tk.StringVar(value="V")
        ttk.Radiobutton(f_add, text="Вертикаль (X)", variable=self.h_type, value="V").grid(row=0, column=0)
        ttk.Radiobutton(f_add, text="Горизонталь (Y)", variable=self.h_type, value="H").grid(row=0, column=1)
        ttk.Button(f_add, text="➕ Создать новую линию", command=self.add_helper).grid(row=0, column=2, padx=20)
        
        h_header = ttk.Frame(frame, padding=(0, 5)); h_header.pack(fill="x")
        # Заголовки колонок
        cols = [("Тип", 30), ("Поз.", 60), ("Старт", 60), ("Конец", 60), ("Текст", 80), ("Метка%", 50), ("Шрифт", 50), ("Слой", 70), ("Толщ", 50)]
        for t, w in cols: ttk.Label(h_header, text=t, width=int(w/7), font='Arial 8 bold').pack(side="left", padx=2)

        self.h_canvas = tk.Canvas(frame)
        self.h_scroll = ttk.Scrollbar(frame, orient="vertical", command=self.h_canvas.yview)
        self.h_frame = ttk.Frame(self.h_canvas)
        self.h_frame.bind("<Configure>", lambda e: self.h_canvas.configure(scrollregion=self.h_canvas.bbox("all")))
        self.h_canvas.create_window((0,0), window=self.h_frame, anchor="nw")
        self.h_canvas.configure(yscrollcommand=self.h_scroll.set)
        self.h_canvas.pack(side="left", fill="both", expand=True); self.h_scroll.pack(side="right", fill="y")

    def add_helper(self):
        x_l, y_l = self.ax.get_xlim(), self.ax.get_ylim()
        is_v = self.h_type.get() == 'V'
        new_h = {
            'type': self.h_type.get(),
            'pos': round(np.mean(x_l if is_v else y_l), 2),
            'start': round(y_l[0] if is_v else x_l[0], 2),
            'end': round(y_l[1] if is_v else x_l[1], 2),
            'color': '#FF0000', 'width': 1.0, 'text': '', 'label_pos': 0.9, 'layer': 'Задний', 'font_size': 9
        }
        self.helper_lines.append(new_h)
        self.update_helper_ui(); self.refresh_plots()

    def update_helper_ui(self):
        for w in self.h_frame.winfo_children(): w.destroy()
        for i, h in enumerate(self.helper_lines):
            f_row = ttk.Frame(self.h_frame); f_row.pack(fill="x", pady=2)
            ttk.Label(f_row, text=h['type'], width=3).pack(side="left")
            
            # Координаты
            for k, w in [('pos', 8), ('start', 8), ('end', 8)]:
                e = ttk.Entry(f_row, width=w); e.insert(0, str(h[k])); e.pack(side="left", padx=1)
                e.bind("<KeyRelease>", lambda ev, ix=i, key=k, en=e: self.edit_helper(ix, key, en.get()))
            
            # Текст
            e_txt = ttk.Entry(f_row, width=10); e_txt.insert(0, h['text']); e_txt.pack(side="left", padx=1)
            e_txt.bind("<KeyRelease>", lambda ev, ix=i, en=e_txt: self.edit_helper(ix, 'text', en.get()))
            
            # Метка % и Размер шрифта
            for k, v_from, v_to in [('label_pos', 0, 100), ('font_size', 5, 30)]:
                val = int(h[k]*100) if k=='label_pos' else h[k]
                sp = tk.Spinbox(f_row, from_=v_from, to=v_to, width=4, command=lambda ix=i, key=k: self.edit_helper_spin(ix, key))
                sp.delete(0, "end"); sp.insert(0, str(val)); sp.pack(side="left", padx=1)
                h[f'{k}_spin'] = sp

            # Слой
            cb_l = ttk.Combobox(f_row, values=["Задний", "Передний"], width=8, state="readonly")
            cb_l.set(h['layer']); cb_l.pack(side="left", padx=1); cb_l.bind("<<ComboboxSelected>>", lambda e, ix=i, c=cb_l: self.edit_helper(ix, 'layer', c.get()))
            
            # ТОЛЩИНА (Width)
            sp_w = tk.Spinbox(f_row, from_=0.1, to=10.0, increment=0.1, width=4, command=lambda ix=i: self.edit_helper_spin(ix, 'width'))
            sp_w.delete(0, "end"); sp_w.insert(0, str(h['width'])); sp_w.pack(side="left", padx=1)
            h['width_spin'] = sp_w

            # Цвет и Удаление
            btn_c = tk.Button(f_row, bg=h['color'], width=2, relief="flat", command=lambda idx=i: self.pick_helper_color(idx)); btn_c.pack(side="left", padx=2)
            ttk.Button(f_row, text="🗑️", width=3, command=lambda idx=i: self.remove_helper(idx)).pack(side="right")

    def edit_helper(self, idx, key, val):
        try:
            if key in ['pos', 'start', 'end']: self.helper_lines[idx][key] = float(val)
            else: self.helper_lines[idx][key] = val
            self.refresh_plots()
        except: pass

    def edit_helper_spin(self, idx, key):
        try:
            spin_val = float(self.helper_lines[idx][f'{key}_spin'].get())
            if key == 'label_pos': self.helper_lines[idx][key] = spin_val / 100.0
            else: self.helper_lines[idx][key] = spin_val
            self.refresh_plots()
        except: pass

    def remove_helper(self, idx):
        self.helper_lines.pop(idx); self.update_helper_ui(); self.refresh_plots()

    def pick_helper_color(self, idx):
        c = colorchooser.askcolor(initialcolor=self.helper_lines[idx]['color'])[1]
        if c: self.helper_lines[idx]['color'] = c; self.update_helper_ui(); self.refresh_plots()

    def draw_helpers(self):
        for artist in self.helper_artists:
            try: artist.remove()
            except: pass
        self.helper_artists = []
        for h in self.helper_lines:
            z = 1 if h['layer'] == 'Задний' else 50
            txt = h['text'] if h['text'].strip() else str(h['pos'])
            if h['type'] == 'V':
                l = self.ax.plot([h['pos'], h['pos']], [h['start'], h['end']], color=h['color'], lw=h['width'], zorder=z)[0]
                ty = h['start'] + (h['end'] - h['start']) * h['label_pos']
                t = self.ax.text(h['pos'], ty, f" {txt}", color=h['color'], fontsize=h['font_size'], va='center', ha='left', fontweight='bold', zorder=z+1)
            else:
                l = self.ax.plot([h['start'], h['end']], [h['pos'], h['pos']], color=h['color'], lw=h['width'], zorder=z)[0]
                tx = h['start'] + (h['end'] - h['start']) * h['label_pos']
                t = self.ax.text(tx, h['pos'], f" {txt}", color=h['color'], fontsize=h['font_size'], va='bottom', ha='center', fontweight='bold', zorder=z+1)
            self.helper_artists.extend([l, t])

    # --- ОСТАЛЬНЫЕ ВКЛАДКИ ---
    def setup_tab_scale(self):
        frame = ttk.Frame(self.tab_scale, padding=20); frame.pack(fill="both")
        ttk.Label(frame, text="Режим:", font='Arial 10 bold').pack(anchor="w")
        ttk.Radiobutton(frame, text="Абсолютный", variable=self.mode_var, value="Абсолютный", command=self.refresh_plots).pack(anchor="w")
        ttk.Radiobutton(frame, text="Нормированный", variable=self.mode_var, value="Нормированный", command=self.refresh_plots).pack(anchor="w")
        ttk.Label(frame, text="Диапазон X (мин):", font='Arial 10 bold').pack(anchor="w", pady=(10,0))
        for a in ['min', 'max']:
            f = ttk.Frame(frame); f.pack(fill="x")
            s = ttk.Scale(f, from_=0, to=1, orient="horizontal", command=self.sync_time_scale)
            s.pack(side="left", fill="x", expand=True)
            e = ttk.Entry(f, width=10); e.pack(side="right", padx=5); e.bind("<Return>", self.sync_time_entry)
            setattr(self, f'scale_{a}', s); setattr(self, f'ent_{a}', e)
        ttk.Label(frame, text="Шаг делений (0=Авто):", font='Arial 10 bold').pack(anchor="w", pady=(10,0))
        for t_type in ['major', 'minor']:
            for ax_name in ['x', 'y']:
                f_t = ttk.Frame(frame); f_t.pack(fill="x")
                ttk.Label(f_t, text=f"{ax_name.upper()} ({t_type}):", width=12).pack(side="left")
                limit = 10 if ax_name == 'x' else 1000000
                s = ttk.Scale(f_t, from_=0, to=limit, command=lambda e, a=ax_name, t=t_type: self.on_tick_change(a, t))
                s.pack(side="left", fill="x", expand=True)
                e = ttk.Entry(f_t, width=10); e.pack(side="right", padx=5); e.insert(0, "0.0")
                e.bind("<Return>", lambda ev, a=ax_name, t=t_type: self.on_tick_entry(a, t))
                setattr(self, f'scale_{t_type}_{ax_name}', s); setattr(self, f'ent_{t_type}_{ax_name}', e)
        ttk.Button(frame, text="🔄 Сброс делений", command=self.reset_ticks).pack(fill="x", pady=10)
        ttk.Button(frame, text="🔍 Подогнать Y (Абс)", command=self.manual_fit).pack(fill="x")

    def setup_tab_files(self):
        f_btns = ttk.Frame(self.tab_files, padding=10); f_btns.pack(fill="x")
        ttk.Button(f_btns, text="➕ Добавить файлы", command=self.add_files).pack(side="left", padx=5)
        ttk.Button(f_btns, text="🗑️ Очистить всё", command=self.clear_all).pack(side="left", padx=5)
        self.f_canvas = tk.Canvas(self.tab_files)
        self.f_scroll = ttk.Scrollbar(self.tab_files, orient="vertical", command=self.f_canvas.yview)
        self.f_frame = ttk.Frame(self.f_canvas)
        self.f_frame.bind("<Configure>", lambda e: self.f_canvas.configure(scrollregion=self.f_canvas.bbox("all")))
        self.f_canvas.create_window((0, 0), window=self.f_frame, anchor="nw")
        self.f_canvas.configure(yscrollcommand=self.f_scroll.set)
        self.f_canvas.pack(side="left", fill="both", expand=True); self.f_scroll.pack(side="right", fill="y")

    def setup_tab_labels(self):
        frame = ttk.Frame(self.tab_labels, padding=20); frame.pack(fill="both")
        for txt, key in [("Заголовок:", 'title_text'), ("Ось X:", 'x_label'), ("Ось Y (Абс):", 'y_label_abs'), ("Ось Y (Норм):", 'y_label_norm')]:
            ttk.Label(frame, text=txt).pack(anchor="w")
            ent = ttk.Entry(frame); ent.insert(0, self.style_params[key]); ent.pack(fill="x", pady=2)
            ent.bind("<KeyRelease>", lambda e, k=key, en=ent: self.on_label_update(k, en))
            self.label_entries[key] = ent
        ttk.Checkbutton(frame, text="Показывать легенду", variable=self.legend_var, command=self.refresh_plots).pack(anchor="w", pady=10)

    def setup_tab_style(self):
        frame = ttk.Frame(self.tab_style, padding=20); frame.pack(fill="both"); frame.columnconfigure(1, weight=1)
        ttk.Label(frame, text="Шрифт:").grid(row=0, column=0, sticky="w")
        self.font_combo = ttk.Combobox(frame, values=["Arial", "Times New Roman", "Verdana", "Tahoma"])
        self.font_combo.set(self.style_params['font_family']); self.font_combo.grid(row=0, column=1, sticky="ew", columnspan=2)
        self.font_combo.bind("<<ComboboxSelected>>", self.update_style)
        rows = [("Заголовок", 'title_size', 8, 24), ("Оси", 'label_size', 8, 20), ("Числа", 'tick_size', 6, 16), 
                ("Легенда", 'legend_size', 6, 16), ("Линии (общ)", 'line_width', 0.5, 5), ("Рамка", 'spine_width', 0.1, 3),
                ("Деления", 'tick_width', 0.1, 3), ("Сетка (толщ)", 'grid_width', 0.1, 2), ("Сетка (прозр)", 'grid_alpha', 0.1, 1)]
        for i, (l, p, f, t) in enumerate(rows): self.create_style_row(frame, l, p, f, t, i+1)
        ttk.Checkbutton(frame, text="Включить сетку", variable=self.grid_var, command=self.update_style).grid(row=11, column=0)
        ttk.Button(frame, text="🔄 Сбросить всё", command=self.reset_all).grid(row=12, column=0, columnspan=3, sticky="ew", pady=10)

    def create_style_row(self, master, label, param, f, t, row):
        ttk.Label(master, text=label+":").grid(row=row, column=0, sticky="w")
        s = ttk.Scale(master, from_=f, to=t, command=lambda e, p=param: self.on_style_slider(p, e))
        s.set(self.style_params[param]); s.grid(row=row, column=1, sticky="ew", padx=5)
        e = ttk.Entry(master, width=6); e.insert(0, f"{self.style_params[param]:.1f}"); e.grid(row=row, column=2)
        e.bind("<Return>", lambda ev, p=param: self.on_style_entry(p))
        self.style_widgets[param] = {'scale': s, 'entry': e}

    def refresh_plots(self):
        if not self.plots_registry or not self.ui_ready: return
        s_t, e_t = self.scale_min.get(), self.scale_max.get()
        self.ax.set_xlim(s_t, e_t if e_t > s_t else s_t+0.01)
        for ax_obj, name in zip([self.ax.xaxis, self.ax.yaxis], ['x', 'y']):
            step = self.style_params[f'{name}_major_step']
            ax_obj.set_major_locator(MultipleLocator(step) if step > 0 else AutoLocator())
            m_step = self.style_params[f'{name}_minor_step']
            ax_obj.set_minor_locator(MultipleLocator(m_step) if m_step > 0 else NullLocator())
        
        if self.mode_var.get() == "Нормированный":
            self.ax.set_ylabel(self.style_params['y_label_norm'])
            for it in self.plots_registry:
                mask = (it['t'] >= s_t) & (it['t'] <= e_t)
                v_y = it['y_orig'][mask]
                it['line'].set_ydata((it['y_orig'] / v_y.max() * 100) if v_y.size and v_y.max()>0 else it['y_orig']*0)
            self.ax.set_ylim(-2, 105)
        else:
            self.ax.set_ylabel(self.style_params['y_label_abs'])
            for it in self.plots_registry: it['line'].set_ydata(it['y_orig'])
            self.ax.set_ylim(-self.global_y_max*0.02, self.global_y_max*1.05)
        
        self.draw_helpers(); self.update_style(); self.canvas.draw_idle()

    def add_files(self):
        paths = filedialog.askopenfilenames(filetypes=[("Data", "*.csv *.txt"), ("All", "*.*")])
        colors = plt.rcParams['axes.prop_cycle'].by_key()['color']
        for p in paths:
            t, y = self.load_data(p)
            if t is not None:
                lbl = os.path.basename(p).split('.')[0]
                line, = self.ax.plot(t, y, label=lbl, lw=self.style_params['line_width'], color=colors[len(self.plots_registry)%len(colors)], zorder=10)
                self.plots_registry.append({'t': t, 'y_orig': y, 'line': line, 'color': line.get_color(), 'label': lbl, 'linestyle': '-', 'line_width': self.style_params['line_width']})
                self.all_data_bounds = [min(self.all_data_bounds[0], t.min()), max(self.all_data_bounds[1], t.max())]
                self.global_y_max = max(self.global_y_max, y.max())
        self.update_ui_list(); self.refresh_plots()

    def update_ui_list(self):
        for w in self.f_frame.winfo_children(): w.destroy()
        for i, item in enumerate(reversed(self.plots_registry)):
            idx = len(self.plots_registry)-1-i
            f = ttk.Frame(self.f_frame); f.pack(fill="x", pady=2, padx=5)
            ttk.Button(f, text="↑", width=2, command=lambda ix=idx: self.move_plot(ix, 1)).pack(side="left")
            ttk.Button(f, text="↓", width=2, command=lambda ix=idx: self.move_plot(ix, -1)).pack(side="left")
            btn_c = tk.Button(f, bg=item['color'], width=2, command=lambda ix=idx: self.pick_color(ix)); btn_c.pack(side="left", padx=2)
            cb = ttk.Combobox(f, values=["-", "--", ":", "-."], width=3, state="readonly"); cb.set(item['linestyle']); cb.pack(side="left")
            cb.bind("<<ComboboxSelected>>", lambda e, ix=idx, c=cb: self.edit_plot(ix, 'linestyle', c.get()))
            sp = tk.Spinbox(f, from_=0.1, to=10, increment=0.1, width=4, command=lambda ix=idx: self.edit_plot(ix, 'line_width', None)); sp.delete(0, "end"); sp.insert(0, f"{item['line_width']:.1f}"); sp.pack(side="left", padx=2); item['width_spin'] = sp
            ent = ttk.Entry(f); ent.insert(0, item['label']); ent.pack(side="left", fill="x", expand=True); ent.bind("<KeyRelease>", lambda e, ix=idx, en=ent: self.edit_plot(ix, 'label', en.get()))
        self.scale_min.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1]); self.scale_max.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
        if self.ui_ready: self.sync_time_scale(None)

    def move_plot(self, idx, d):
        if 0 <= idx+d < len(self.plots_registry):
            self.plots_registry[idx], self.plots_registry[idx+d] = self.plots_registry[idx+d], self.plots_registry[idx]
            for i, it in enumerate(self.plots_registry): it['line'].set_zorder(i+10)
            self.update_ui_list(); self.refresh_plots()

    def edit_plot(self, idx, key, val):
        if key == 'line_width': val = float(self.plots_registry[idx]['width_spin'].get())
        self.plots_registry[idx][key] = val
        if key == 'linestyle': self.plots_registry[idx]['line'].set_linestyle(val)
        if key == 'label': self.plots_registry[idx]['line'].set_label(val)
        self.refresh_plots()

    def update_style(self, event=None):
        if not self.ui_ready: return
        p = self.style_params
        plt.rcParams['font.family'] = self.font_combo.get()
        self.ax.set_title(p['title_text'], fontsize=p['title_size'])
        self.ax.set_xlabel(p['x_label'], fontsize=p['label_size'])
        self.ax.tick_params(labelsize=p['tick_size'], width=p['tick_width'], which='both')
        for s in self.ax.spines.values(): s.set_linewidth(p['spine_width'])
        for it in self.plots_registry: it['line'].set_linewidth(it['line_width'])
        self.ax.grid(False); self.ax.grid(self.grid_var.get(), alpha=p['grid_alpha'], lw=p['grid_width'])
        if self.ax.get_legend(): self.ax.get_legend().remove()
        if self.legend_var.get() and self.plots_registry: self.ax.legend(fontsize=p['legend_size'])

    def on_style_slider(self, p, v):
        if not self.ui_ready or p not in self.style_widgets: return
        self.style_params[p] = float(v); self.style_widgets[p]['entry'].delete(0, tk.END); self.style_widgets[p]['entry'].insert(0, f"{float(v):.1f}")
        if p == 'line_width': 
            for it in self.plots_registry: it['line_width'] = float(v)
        self.refresh_plots()

    def on_style_entry(self, p):
        try: v = float(self.style_widgets[p]['entry'].get()); self.style_widgets[p]['scale'].set(v); self.style_params[p] = v; self.refresh_plots()
        except: pass

    def on_tick_change(self, ax, ty):
        v = getattr(self, f'scale_{ty}_{ax}').get(); self.style_params[f'{ax}_{ty}_step'] = v
        getattr(self, f'ent_{ty}_{ax}').delete(0, tk.END); getattr(self, f'ent_{ty}_{ax}').insert(0, f"{v:.3f}"); self.refresh_plots()

    def on_tick_entry(self, ax, ty):
        try: v = float(getattr(self, f'ent_{ty}_{ax}').get()); getattr(self, f'scale_{ty}_{ax}').set(v); self.style_params[f'{ax}_{ty}_step'] = v; self.refresh_plots()
        except: pass

    def reset_ticks(self):
        for a in ['x','y']:
            for t in ['major','minor']:
                self.style_params[f'{a}_{t}_step'] = 0; getattr(self, f'scale_{t}_{a}').set(0); getattr(self, f'ent_{t}_{a}').delete(0, tk.END); getattr(self, f'ent_{t}_{a}').insert(0, "0.0")
        self.refresh_plots()

    def sync_time_scale(self, e):
        if not self.ui_ready: return
        for a in ['min','max']: getattr(self, f'ent_{a}').delete(0, tk.END); getattr(self, f'ent_{a}').insert(0, f"{getattr(self, f'scale_{a}').get():.3f}")
        self.refresh_plots()

    def sync_time_entry(self, e):
        try: self.scale_min.set(float(self.ent_min.get())); self.scale_max.set(float(self.ent_max.get())); self.refresh_plots()
        except: pass

    def on_label_update(self, k, en): self.style_params[k] = en.get(); self.refresh_plots()
    def pick_color(self, idx):
        c = colorchooser.askcolor(initialcolor=self.plots_registry[idx]['color'])[1]
        if c: self.plots_registry[idx]['color'] = c; self.plots_registry[idx]['line'].set_color(c); self.update_ui_list(); self.refresh_plots()
    def pick_helper_color(self, idx):
        c = colorchooser.askcolor(initialcolor=self.helper_lines[idx]['color'])[1]
        if c: self.helper_lines[idx]['color'] = c; self.update_helper_ui(); self.refresh_plots()
    def reset_all(self):
        self.style_params = self.default_style.copy(); self.grid_var.set(True); self.legend_var.set(True); self.update_ui_list(); self.refresh_plots()
    def manual_fit(self):
        if self.mode_var.get() == "Абсолютный":
            s, e = self.scale_min.get(), self.scale_max.get(); m = 0
            for it in self.plots_registry:
                v = it['y_orig'][(it['t']>=s)&(it['t']<=e)]
                if v.size: m = max(m, v.max())
            if m > 0: self.ax.set_ylim(-m*0.02, m*1.05); self.canvas.draw()
    def load_data(self, p):
        try:
            df = pd.read_csv(p, sep=';', decimal=',', comment='#', header=None)
            return pd.to_numeric(df.iloc[:, 1], errors='coerce').values, pd.to_numeric(df.iloc[:, 2], errors='coerce').values
        except: return None, None
    def clear_all(self):
        for it in self.plots_registry: it['line'].remove()
        self.plots_registry = []; self.global_y_max = 0; self.all_data_bounds = [float('inf'), float('-inf')]; self.ax.set_xlim(0, 1); self.ax.set_ylim(0, 1); self.update_ui_list(); self.canvas.draw()
    def save_svg(self):
        p = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        if p: self.fig.savefig(p, format='svg', bbox_inches='tight')
    def on_close(self): plt.close('all'); self.root.quit(); self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk(); app = ChromatogramApp(root); root.mainloop()