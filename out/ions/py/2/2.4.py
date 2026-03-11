import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.ticker import MultipleLocator, AutoLocator, NullLocator
import tkinter as tk
from tkinter import filedialog, ttk, colorchooser
import os
import numpy as np
import json

class ChromatogramApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Chromatogram Professional Viewer")
        self.ui_ready = False
        
        # 1. Дефолтные настройки
        self.default_style = {
            'gen_font': 'Arial', 'gen_weight': 'normal', 'gen_style': 'normal',
            'title_font': 'Inherit', 'title_weight': 'Inherit', 'title_style': 'Inherit', 'title_size': 14.0,
            'label_font': 'Inherit', 'label_weight': 'Inherit', 'label_style': 'Inherit', 'label_size': 12.0,
            'tick_font': 'Inherit', 'tick_weight': 'Inherit', 'tick_style': 'Inherit', 'tick_size': 10.0,
            'legend_font': 'Inherit', 'legend_weight': 'Inherit', 'legend_style': 'Inherit', 'legend_size': 10.0,
            'line_width': 1.5, 'grid_alpha': 0.3, 'grid_width': 0.8, 'spine_width': 1.0,
            'major_tick_width': 1.2, 'major_tick_length': 5.0,
            'minor_tick_width': 0.8, 'minor_tick_length': 3.0,
            'x_major_step': 0.0, 'y_major_step': 0.0, 'x_minor_step': 0.0, 'y_minor_step': 0.0,
            'title_text': 'EIC Chromatograms', 'x_label': 'Время (мин)',
            'y_label_abs': 'Интенсивность (Counts)', 'y_label_norm': 'Относительная интенсивность (%)',
            'show_legend': True
        }
        self.style_params = self.default_style.copy()
        
        # 2. Данные
        self.plots_registry = []
        self.helper_lines = []
        self.helper_artists = []
        self.all_data_bounds = [float('inf'), float('-inf')]
        self.global_y_max = 0
        self.style_widgets = {}
        self.label_entries = {}
        self.tick_widgets = {}
        self.font_vars = {}
        
        self.legend_var = tk.BooleanVar(value=self.style_params['show_legend'])
        self.mode_var = tk.StringVar(value="Абсолютный")
        
        # 3. Matplotlib
        self.fig, self.ax = plt.subplots(figsize=(10, 6))
        self.fig.subplots_adjust(bottom=0.2, left=0.12, top=0.9, right=0.95)
        self.canvas = FigureCanvasTkAgg(self.fig, master=self.root)
        self.canvas.get_tk_widget().pack(side=tk.TOP, fill=tk.BOTH, expand=1)
        
        # 4. Интерфейс
        self.create_control_window()
        self.root.protocol("WM_DELETE_WINDOW", self.on_close)
        self.ui_ready = True

    def get_font_prop(self, prefix, prop_type):
        val = self.font_vars[f'{prefix}_{prop_type}'].get()
        return self.font_vars[f'gen_{prop_type}'].get() if val == "Inherit" else val

    def create_control_window(self):
        self.ctrl_win = tk.Toplevel(self.root)
        self.ctrl_win.title("Панель управления")
        self.ctrl_win.geometry("1150x950")
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

    def setup_tab_style(self):
        container = ttk.Frame(self.tab_style); container.pack(fill="both", expand=True)
        canvas = tk.Canvas(container); scrollbar = ttk.Scrollbar(container, orient="vertical", command=canvas.yview)
        scroll_frame = ttk.Frame(canvas); scroll_frame.bind("<Configure>", lambda e: canvas.configure(scrollregion=canvas.bbox("all")))
        canvas.create_window((0, 0), window=scroll_frame, anchor="nw"); canvas.configure(yscrollcommand=scrollbar.set)
        canvas.pack(side="left", fill="both", expand=True); scrollbar.pack(side="right", fill="y")
        f = ttk.Frame(scroll_frame, padding=20); f.pack(fill="both"); f.columnconfigure(1, weight=1)
        
        io_frame = ttk.Frame(f); io_frame.grid(row=0, column=0, columnspan=3, sticky="ew", pady=(0,15))
        ttk.Button(io_frame, text="💾 Экспорт стиля", command=self.export_style).pack(side="left", expand=True, fill="x", padx=2)
        ttk.Button(io_frame, text="📂 Импорт стиля", command=self.import_style).pack(side="left", expand=True, fill="x", padx=2)

        self.create_font_row(f, "Общий шрифт", "gen", 2, include_inherit=False)
        self.create_font_row(f, "Заголовок", "title", 5, size_range=(8, 30))
        self.create_font_row(f, "Подписи осей", "label", 7, size_range=(8, 24))
        self.create_font_row(f, "Числа на осях", "tick", 9, size_range=(6, 20))
        self.create_font_row(f, "Легенда", "legend", 11, size_range=(6, 20))

        style_rows = [
            ("Линии графиков", 'line_width', 0.5, 5), ("Толщина рамки", 'spine_width', 0.1, 3),
            ("Осн. деления (толщ)", 'major_tick_width', 0.1, 3), ("Осн. деления (длина)", 'major_tick_length', 0, 15),
            ("Пром. деления (толщ)", 'minor_tick_width', 0.1, 3), ("Пром. деления (длина)", 'minor_tick_length', 0, 15),
            ("Сетка (толщина)", 'grid_width', 0.1, 2), ("Сетка (прозрачность)", 'grid_alpha', 0, 1)
        ]
        r_idx = 14
        for lbl, p, f_val, t_val in style_rows:
            self.create_style_row(f, lbl, p, f_val, t_val, r_idx)
            r_idx += 1

        ttk.Separator(f, orient="horizontal").grid(row=r_idx, column=0, columnspan=3, sticky="ew", pady=15); r_idx += 1
        tick_rows = [("X Основной", 'x_major_step', 10), ("Y Основной", 'y_major_step', 1000000),
                     ("X Промежут.", 'x_minor_step', 5), ("Y Промежут.", 'y_minor_step', 500000)]
        for lbl, p, lim in tick_rows:
            ttk.Label(f, text=lbl+":").grid(row=r_idx, column=0, sticky="w")
            s = ttk.Scale(f, from_=0, to=lim, command=lambda e, param=p: self.on_tick_slider(param, e))
            s.set(self.style_params[p]); s.grid(row=r_idx, column=1, sticky="ew", padx=5)
            e = ttk.Entry(f, width=8); e.insert(0, "0.0"); e.grid(row=r_idx, column=2); e.bind("<Return>", lambda ev, param=p: self.on_tick_entry(param))
            self.tick_widgets[p] = {'scale': s, 'entry': e}; r_idx += 1
        ttk.Button(f, text="🔄 Сбросить всё", command=self.reset_all).grid(row=r_idx, column=0, columnspan=3, sticky="ew", pady=20)

    def create_font_row(self, master, label, prefix, row, include_inherit=True, size_range=None):
        ttk.Label(master, text=label+":", font='Arial 9 bold').grid(row=row, column=0, sticky="w", pady=(5,0))
        f_frame = ttk.Frame(master); f_frame.grid(row=row+1, column=0, columnspan=3, sticky="ew")
        families = ["Arial", "Times New Roman", "Verdana", "Courier New", "Tahoma"]
        weights = ["normal", "bold", "light"]; styles = ["normal", "italic"]
        if include_inherit: families, weights, styles = [["Inherit"] + x for x in [families, weights, styles]]
        for prop, vals, w in [('font', families, 15), ('weight', weights, 10), ('style', styles, 10)]:
            var = tk.StringVar(value=self.style_params[f'{prefix}_{prop}'])
            cb = ttk.Combobox(f_frame, textvariable=var, values=vals, width=w, state="readonly"); cb.pack(side="left", padx=2)
            self.font_vars[f'{prefix}_{prop}'] = var; var.trace_add("write", lambda *a: self.refresh_plots())
        if size_range:
            p_name = f'{prefix}_size'
            s = ttk.Scale(f_frame, from_=size_range[0], to=size_range[1], command=lambda e: self.on_style_slider(p_name, e))
            s.set(self.style_params[p_name]); s.pack(side="left", fill="x", expand=True, padx=5)
            e = ttk.Entry(f_frame, width=4); e.insert(0, f"{self.style_params[p_name]:.1f}"); e.pack(side="left"); e.bind("<Return>", lambda ev: self.on_style_entry(p_name))
            self.style_widgets[p_name] = {'scale': s, 'entry': e}

    def create_style_row(self, master, label, param, f, t, row):
        ttk.Label(master, text=label+":").grid(row=row, column=0, sticky="w")
        s = ttk.Scale(master, from_=f, to=t, command=lambda e: self.on_style_slider(param, e))
        s.set(self.style_params[param]); s.grid(row=row, column=1, sticky="ew", padx=5)
        e = ttk.Entry(master, width=6); e.insert(0, f"{self.style_params[param]:.1f}"); e.grid(row=row, column=2); e.bind("<Return>", lambda ev: self.on_style_entry(param))
        self.style_widgets[param] = {'scale': s, 'entry': e}

    def update_style(self, event=None):
        if not self.ui_ready: return
        p = self.style_params
        self.ax.set_title(p['title_text'], fontfamily=self.get_font_prop('title', 'font'), fontweight=self.get_font_prop('title', 'weight'), fontstyle=self.get_font_prop('title', 'style'), fontsize=p['title_size'])
        y_label_text = p['y_label_norm'] if self.mode_var.get() == "Нормированный" else p['y_label_abs']
        self.ax.set_xlabel(p['x_label'], fontfamily=self.get_font_prop('label', 'font'), fontweight=self.get_font_prop('label', 'weight'), fontstyle=self.get_font_prop('label', 'style'), fontsize=p['label_size'])
        self.ax.set_ylabel(y_label_text, fontfamily=self.get_font_prop('label', 'font'), fontweight=self.get_font_prop('label', 'weight'), fontstyle=self.get_font_prop('label', 'style'), fontsize=p['label_size'])
        for ax_obj in [self.ax.xaxis, self.ax.yaxis]:
            for tick in ax_obj.get_ticklabels():
                tick.set_fontfamily(self.get_font_prop('tick', 'font')), tick.set_fontweight(self.get_font_prop('tick', 'weight')), tick.set_fontstyle(self.get_font_prop('tick', 'style')), tick.set_fontsize(p['tick_size'])
        self.ax.tick_params(which='major', width=p['major_tick_width'], length=p['major_tick_length'])
        self.ax.tick_params(which='minor', width=p['minor_tick_width'], length=p['minor_tick_length'])
        for s in self.ax.spines.values(): s.set_linewidth(p['spine_width'])
        for it in self.plots_registry: it['line'].set_linewidth(it['line_width'])
        self.ax.grid(False); self.ax.grid(visible=True, which='major', alpha=p['grid_alpha'], lw=p['grid_width'])
        if self.ax.get_legend(): self.ax.get_legend().remove()
        if self.legend_var.get() and self.plots_registry:
            leg = self.ax.legend(fontsize=p['legend_size'])
            plt.setp(leg.get_texts(), fontfamily=self.get_font_prop('legend', 'font'), fontweight=self.get_font_prop('legend', 'weight'), fontstyle=self.get_font_prop('legend', 'style'))
        self.canvas.draw_idle()

    def refresh_plots(self):
        if not self.plots_registry or not self.ui_ready: return
        s_t, e_t = self.scale_min.get(), self.scale_max.get()
        self.ax.set_xlim(s_t, e_t if e_t > s_t else s_t+0.01)
        
        # ... (код настройки осей остается прежним) ...
        for ax_obj, name in zip([self.ax.xaxis, self.ax.yaxis], ['x', 'y']):
            major = self.style_params[f'{name}_major_step']
            ax_obj.set_major_locator(MultipleLocator(major) if major > 0 else AutoLocator())
            minor = self.style_params[f'{name}_minor_step']
            ax_obj.set_minor_locator(MultipleLocator(minor) if minor > 0 else NullLocator())

        # ... (код нормировки и отрисовки линий остается прежним) ...
        if self.mode_var.get() == "Нормированный":
            for it in self.plots_registry:
                mask = (it['t'] >= s_t) & (it['t'] <= e_t)
                v_y = it['y_orig'][mask]
                it['line'].set_ydata((it['y_orig'] / v_y.max() * 100) if v_y.size and v_y.max()>0 else it['y_orig']*0)
            self.ax.set_ylim(-2, 105)
        else:
            for it in self.plots_registry: it['line'].set_ydata(it['y_orig'])
            self.ax.set_ylim(-self.global_y_max*0.02, self.global_y_max*1.05)

        for it in self.plots_registry: it['line'].set_label(it['label'] if it['show_in_legend'] else "_nolegend_")

        # ОТРИСОВКА ВСПОМОГАТЕЛЬНЫХ ЛИНИЙ (ОБНОВЛЕНО)
        for a in self.helper_artists: a.remove()
        self.helper_artists = []
        for h in self.helper_lines:
            z = 1 if h['layer'] == 'Задний' else 50
            txt = h['text'] if h['text'].strip() else str(h['pos'])
            
            if h['type'] == 'V':
                l = self.ax.plot([h['pos'], h['pos']], [h['start'], h['end']], color=h['color'], lw=h['width'], zorder=z)[0]
                # X = позиция + смещение, Y = расчет по label_pos
                tx = h['pos'] + h.get('label_offset', 0)
                ty = h['start'] + (h['end'] - h['start']) * h['label_pos']
                t = self.ax.text(tx, ty, txt, color=h['color'], fontsize=h['font_size'], 
                                 va='center', ha='center', fontweight='bold', zorder=z+1)
            else:
                l = self.ax.plot([h['start'], h['end']], [h['pos'], h['pos']], color=h['color'], lw=h['width'], zorder=z)[0]
                # Y = позиция + смещение, X = расчет по label_pos
                ty = h['pos'] + h.get('label_offset', 0)
                tx = h['start'] + (h['end'] - h['start']) * h['label_pos']
                t = self.ax.text(tx, ty, txt, color=h['color'], fontsize=h['font_size'], 
                                 va='center', ha='center', fontweight='bold', zorder=z+1)
            self.helper_artists.extend([l, t])
        
        self.update_style()

    def setup_tab_files(self):
        f_btns = ttk.Frame(self.tab_files, padding=10); f_btns.pack(fill="x")
        ttk.Button(f_btns, text="➕ Добавить файлы", command=self.add_files).pack(side="left", padx=5)
        ttk.Button(f_btns, text="🗑️ Очистить всё", command=self.clear_all).pack(side="left", padx=5)
        h_header = ttk.Frame(self.tab_files, padding=(15, 0)); h_header.pack(fill="x")
        cols = [("Слой", 60), ("Цвет", 40), ("Стиль", 50), ("Толщ", 50), ("Лег", 40), ("Имя в легенде", 200)]
        for t, w in cols: ttk.Label(h_header, text=t, width=int(w/7), font='Arial 8 bold').pack(side="left", padx=2)
        self.f_canvas = tk.Canvas(self.tab_files); self.f_scroll = ttk.Scrollbar(self.tab_files, orient="vertical", command=self.f_canvas.yview)
        self.f_frame = ttk.Frame(self.f_canvas); self.f_frame.bind("<Configure>", lambda e: self.f_canvas.configure(scrollregion=self.f_canvas.bbox("all")))
        self.f_canvas.create_window((0, 0), window=self.f_frame, anchor="nw"); self.f_canvas.configure(yscrollcommand=self.f_scroll.set)
        self.f_canvas.pack(side="left", fill="both", expand=True); self.f_scroll.pack(side="right", fill="y")

    def setup_tab_scale(self):
        frame = ttk.Frame(self.tab_scale, padding=20); frame.pack(fill="both")
        ttk.Label(frame, text="Режим:", font='Arial 10 bold').pack(anchor="w")
        ttk.Radiobutton(frame, text="Абсолютный", variable=self.mode_var, value="Абсолютный", command=self.refresh_plots).pack(anchor="w")
        ttk.Radiobutton(frame, text="Нормированный", variable=self.mode_var, value="Нормированный", command=self.refresh_plots).pack(anchor="w")
        for a in ['min', 'max']:
            f = ttk.Frame(frame); f.pack(fill="x")
            s = ttk.Scale(f, from_=0, to=1, orient="horizontal", command=self.sync_time_scale)
            s.pack(side="left", fill="x", expand=True)
            e = ttk.Entry(f, width=10); e.pack(side="right", padx=5); e.bind("<Return>", self.sync_time_entry)
            setattr(self, f'scale_{a}', s); setattr(self, f'ent_{a}', e)
        ttk.Button(frame, text="🔍 Подогнать Y", command=self.manual_fit).pack(fill="x", pady=20)

    def setup_tab_labels(self):
        frame = ttk.Frame(self.tab_labels, padding=20); frame.pack(fill="both")
        for txt, key in [("Заголовок:", 'title_text'), ("Ось X:", 'x_label'), ("Ось Y (Абс):", 'y_label_abs'), ("Ось Y (Норм):", 'y_label_norm')]:
            ttk.Label(frame, text=txt).pack(anchor="w")
            ent = ttk.Entry(frame); ent.insert(0, self.style_params[key]); ent.pack(fill="x", pady=2); ent.bind("<KeyRelease>", lambda e, k=key, en=ent: self.on_label_update(k, en))
            self.label_entries[key] = ent
        ttk.Checkbutton(frame, text="Легенда", variable=self.legend_var, command=self.refresh_plots).pack(anchor="w", pady=10)

    def setup_tab_helpers(self):
        frame = ttk.Frame(self.tab_helpers, padding=10); frame.pack(fill="both", expand=True)
        f_add = ttk.LabelFrame(frame, text="Добавить линию", padding=10); f_add.pack(fill="x")
        self.h_type = tk.StringVar(value="V")
        ttk.Radiobutton(f_add, text="Вертикаль", variable=self.h_type, value="V").grid(row=0, column=0)
        ttk.Radiobutton(f_add, text="Горизонталь", variable=self.h_type, value="H").grid(row=0, column=1)
        ttk.Button(f_add, text="➕ Создать", command=self.add_helper).grid(row=0, column=2, padx=20)
        self.h_canvas = tk.Canvas(frame); self.h_scroll = ttk.Scrollbar(frame, orient="vertical", command=self.h_canvas.yview)
        self.h_frame = ttk.Frame(self.h_canvas); self.h_frame.bind("<Configure>", lambda e: self.h_canvas.configure(scrollregion=self.h_canvas.bbox("all")))
        self.h_canvas.create_window((0,0), window=self.h_frame, anchor="nw"); self.h_canvas.configure(yscrollcommand=self.h_scroll.set)
        self.h_canvas.pack(side="left", fill="both", expand=True); self.h_scroll.pack(side="right", fill="y")

    def add_helper(self):
        x_l, y_l = self.ax.get_xlim(), self.ax.get_ylim()
        is_v = self.h_type.get() == 'V'
        new_h = {
            'type': self.h_type.get(), 
            'pos': round(np.mean(x_l if is_v else y_l), 2), 
            'start': round(y_l[0] if is_v else x_l[0], 2), 
            'end': round(y_l[1] if is_v else x_l[1], 2), 
            'color': '#FF0000', 
            'width': 1.0, 
            'text': '', 
            'label_pos': 0.5,      # Позиция вдоль линии (0-1)
            'label_offset': 0.0,   # Смещение ПЕРПЕНДИКУЛЯРНО линии (новое)
            'layer': 'Задний', 
            'font_size': 9
        }
        self.helper_lines.append(new_h)
        self.update_helper_ui()
        self.refresh_plots()

    def update_helper_ui(self):
        for w in self.h_frame.winfo_children(): w.destroy()
        for i, h in enumerate(self.helper_lines):
            f_row = ttk.Frame(self.h_frame); f_row.pack(fill="x", pady=2)
            
            ttk.Label(f_row, text=h['type'], width=3).pack(side="left")
            
            # 1. Поля координат (pos, start, end)
            for k, w in [('pos', 7), ('start', 7), ('end', 7)]:
                e = ttk.Entry(f_row, width=w)
                e.insert(0, str(h[k]))
                e.pack(side="left", padx=1)
                # Привязки:
                e.bind("<KeyRelease>", lambda ev, ix=i, key=k, en=e: self.edit_helper(ix, key, en.get()))
                e.bind("<Return>", lambda ev: self.root.focus_set())
                e.bind("<FocusOut>", lambda ev: self.root.focus_set())
            
            # 2. Поле текста
            e_txt = ttk.Entry(f_row, width=10)
            e_txt.insert(0, h['text'])
            e_txt.pack(side="left", padx=1)
            e_txt.bind("<KeyRelease>", lambda ev, ix=i, en=e_txt: self.edit_helper(ix, 'text', en.get()))
            e_txt.bind("<Return>", lambda ev: self.root.focus_set())
            
            # 3. Спинбоксы (Позиция %, Смещение, Шрифт, Толщина)
            # Для каждого спинбокса добавляем обработку Enter и потери фокуса
            
            # Позиция %
            sp_p = tk.Spinbox(f_row, from_=0, to=100, width=4, command=lambda ix=i: self.edit_helper_spin(ix, 'label_pos'))
            sp_p.delete(0, "end"); sp_p.insert(0, str(int(h['label_pos']*100))); sp_p.pack(side="left", padx=1)
            h['label_pos_spin'] = sp_p
            sp_p.bind("<Return>", lambda ev, ix=i: [self.edit_helper_spin(ix, 'label_pos'), self.root.focus_set()])
            sp_p.bind("<FocusOut>", lambda ev, ix=i: self.edit_helper_spin(ix, 'label_pos'))

            # Смещение
            sp_o = tk.Spinbox(f_row, from_=-1000000, to=1000000, increment=0.1, width=5, command=lambda ix=i: self.edit_helper_spin(ix, 'label_offset'))
            sp_o.delete(0, "end"); sp_o.insert(0, str(h.get('label_offset', 0))); sp_o.pack(side="left", padx=1)
            h['label_offset_spin'] = sp_o
            sp_o.bind("<Return>", lambda ev, ix=i: [self.edit_helper_spin(ix, 'label_offset'), self.root.focus_set()])
            sp_o.bind("<FocusOut>", lambda ev, ix=i: self.edit_helper_spin(ix, 'label_offset'))

            # Шрифт
            sp_f = tk.Spinbox(f_row, from_=5, to=30, width=3, command=lambda ix=i: self.edit_helper_spin(ix, 'font_size'))
            sp_f.delete(0, "end"); sp_f.insert(0, str(h['font_size'])); sp_f.pack(side="left", padx=1)
            h['font_size_spin'] = sp_f
            sp_f.bind("<Return>", lambda ev, ix=i: [self.edit_helper_spin(ix, 'font_size'), self.root.focus_set()])
            sp_f.bind("<FocusOut>", lambda ev, ix=i: self.edit_helper_spin(ix, 'font_size'))

            # Слой
            cb_l = ttk.Combobox(f_row, values=["Задний", "Передний"], width=8, state="readonly"); cb_l.set(h['layer']); cb_l.pack(side="left", padx=1)
            cb_l.bind("<<ComboboxSelected>>", lambda e, ix=i, c=cb_l: self.edit_helper(ix, 'layer', c.get()))
            
            # Толщина линии
            sp_w = tk.Spinbox(f_row, from_=0.1, to=10.0, increment=0.1, width=4, command=lambda ix=i: self.edit_helper_spin(ix, 'width'))
            sp_w.delete(0, "end"); sp_w.insert(0, str(h['width'])); sp_w.pack(side="left", padx=1)
            h['width_spin'] = sp_w
            sp_w.bind("<Return>", lambda ev, ix=i: [self.edit_helper_spin(ix, 'width'), self.root.focus_set()])
            sp_w.bind("<FocusOut>", lambda ev, ix=i: self.edit_helper_spin(ix, 'width'))
            
            # Кнопки цвета и удаления
            btn_c = tk.Button(f_row, bg=h['color'], width=2, relief="flat", command=lambda idx=i: self.pick_helper_color(idx))
            btn_c.pack(side="left", padx=2)
            ttk.Button(f_row, text="🗑️", width=3, command=lambda idx=i: self.remove_helper(idx)).pack(side="right")

    def edit_helper(self, idx, key, val):
        try:
            if key in ['pos', 'start', 'end']:
                # Очистка ввода: убираем пробелы и меняем запятую на точку
                clean_val = str(val).replace(',', '.').strip()
                if not clean_val: return # Если поле пустое, ничего не делаем
                self.helper_lines[idx][key] = float(clean_val)
            else:
                self.helper_lines[idx][key] = val
            
            # Принудительно перерисовываем
            self.refresh_plots()
        except ValueError:
            # Если введено совсем не число (например, буквы), просто игнорируем
            pass

    def edit_helper_spin(self, idx, key):
        try:
            # Получаем текст из спинбокса, чистим его
            raw_val = self.helper_lines[idx][f'{key}_spin'].get().replace(',', '.').strip()
            spin_val = float(raw_val)
            
            if key == 'label_pos': 
                self.helper_lines[idx][key] = spin_val / 100.0
            else: 
                self.helper_lines[idx][key] = spin_val
            
            self.refresh_plots()
        except ValueError:
            pass

    def remove_helper(self, idx): self.helper_lines.pop(idx); self.update_helper_ui(); self.refresh_plots()
    def pick_helper_color(self, idx):
        c = colorchooser.askcolor(initialcolor=self.helper_lines[idx]['color'])[1]
        if c: self.helper_lines[idx]['color'] = c; self.update_helper_ui(); self.refresh_plots()

    def add_files(self):
        paths = filedialog.askopenfilenames(filetypes=[("Data", "*.csv *.txt"), ("All", "*.*")])
        if not paths: return
        colors = plt.rcParams['axes.prop_cycle'].by_key()['color']
        for p in paths:
            t, y = self.load_data(p)
            if t is not None:
                lbl = os.path.basename(p).split('.')[0]
                line, = self.ax.plot(t, y, label=lbl, lw=self.style_params['line_width'], color=colors[len(self.plots_registry)%len(colors)], zorder=10)
                self.plots_registry.append({'t': t, 'y_orig': y, 'line': line, 'color': line.get_color(), 'label': lbl, 'linestyle': '-', 'line_width': self.style_params['line_width'], 'show_in_legend': True})
                self.all_data_bounds = [min(self.all_data_bounds[0], t.min()), max(self.all_data_bounds[1], t.max())]
                self.global_y_max = max(self.global_y_max, y.max())
        
        # ИСПРАВЛЕНО: Установка интервала X по загруженным данным
        if self.all_data_bounds[0] != float('inf'):
            self.scale_min.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
            self.scale_max.config(from_=self.all_data_bounds[0], to=self.all_data_bounds[1])
            self.scale_min.set(self.all_data_bounds[0])
            self.scale_max.set(self.all_data_bounds[1])
            self.sync_time_scale(None)
        self.update_ui_list(); self.refresh_plots()

    def update_ui_list(self):
        for w in self.f_frame.winfo_children(): w.destroy()
        for i, item in enumerate(reversed(self.plots_registry)):
            idx = len(self.plots_registry)-1-i
            f = ttk.Frame(self.f_frame); f.pack(fill="x", pady=2, padx=5)
            
            # Кнопки порядка
            ttk.Button(f, text="↑", width=2, command=lambda ix=idx: self.move_plot(ix, 1)).pack(side="left")
            ttk.Button(f, text="↓", width=2, command=lambda ix=idx: self.move_plot(ix, -1)).pack(side="left", padx=(0,5))
            
            # Цвет
            btn_c = tk.Button(f, bg=item['color'], width=2, command=lambda ix=idx: self.pick_color(ix)); btn_c.pack(side="left", padx=2)
            
            # Стиль линии
            cb = ttk.Combobox(f, values=["-", "--", ":", "-."], width=3, state="readonly"); cb.set(item['linestyle']); cb.pack(side="left", padx=2)
            cb.bind("<<ComboboxSelected>>", lambda e, ix=idx, c=cb: self.edit_plot(ix, 'linestyle', c.get()))
            
            # ТОЛЩИНА ЛИНИИ (Spinbox)
            sp = tk.Spinbox(f, from_=0.1, to=10, increment=0.1, width=4, command=lambda ix=idx: self.edit_plot(ix, 'line_width', None))
            sp.delete(0, "end"); sp.insert(0, f"{item['line_width']:.1f}"); sp.pack(side="left", padx=2)
            item['width_spin'] = sp
            # Привязки для ручного ввода толщины:
            sp.bind("<Return>", lambda e, ix=idx: [self.edit_plot(ix, 'line_width', None), self.root.focus_set()])
            sp.bind("<FocusOut>", lambda e, ix=idx: self.edit_plot(ix, 'line_width', None))
            
            # Легенда
            leg_v = tk.BooleanVar(value=item['show_in_legend'])
            ttk.Checkbutton(f, variable=leg_v, command=lambda ix=idx, v=leg_v: self.edit_plot(ix, 'show_in_legend', v.get())).pack(side="left", padx=5)
            
            # Кнопка удаления
            ttk.Button(f, text="🗑️", width=3, command=lambda ix=idx: self.remove_plot(ix)).pack(side="right", padx=2)
            
            # НАЗВАНИЕ (Entry)
            ent = ttk.Entry(f); ent.insert(0, item['label']); ent.pack(side="left", fill="x", expand=True)
            # Привязки для названия:
            ent.bind("<KeyRelease>", lambda e, ix=idx, en=ent: self.edit_plot(ix, 'label', en.get()))
            ent.bind("<Return>", lambda e: self.root.focus_set())
            ent.bind("<FocusOut>", lambda e: self.root.focus_set())

    def move_plot(self, idx, d):
        if 0 <= idx+d < len(self.plots_registry):
            self.plots_registry[idx], self.plots_registry[idx+d] = self.plots_registry[idx+d], self.plots_registry[idx]
            for i, it in enumerate(self.plots_registry): it['line'].set_zorder(i+10)
            self.update_ui_list(); self.refresh_plots()

    def remove_plot(self, idx):
        # 1. Удаляем линию с самого графика Matplotlib
        item = self.plots_registry.pop(idx)
        item['line'].remove()
        
        # 2. Если это был последний файл, сбрасываем границы
        if not self.plots_registry:
            self.global_y_max = 0
            self.all_data_bounds = [float('inf'), float('-inf')]
            self.ax.set_xlim(0, 1)
            self.ax.set_ylim(0, 1)
        
        # 3. Обновляем интерфейс и перерисовываем холст
        self.update_ui_list()
        self.refresh_plots()

    def edit_plot(self, idx, key, val):
        try:
            if key == 'line_width':
                # Получаем значение из спинбокса, чистим от пробелов и запятых
                raw_val = self.plots_registry[idx]['width_spin'].get().replace(',', '.').strip()
                if not raw_val: return
                val = float(raw_val)
            
            self.plots_registry[idx][key] = val
            
            if key == 'linestyle': 
                self.plots_registry[idx]['line'].set_linestyle(val)
            
            self.refresh_plots()
        except ValueError:
            pass

    def on_style_slider(self, p, v):
        if not self.ui_ready or p not in self.style_widgets: return
        val = float(v); self.style_params[p] = val
        self.style_widgets[p]['entry'].delete(0, tk.END); self.style_widgets[p]['entry'].insert(0, f"{val:.1f}")
        if p == 'line_width': 
            for it in self.plots_registry: it['line_width'] = val
        self.refresh_plots()

    def on_style_entry(self, p):
        try: v = float(self.style_widgets[p]['entry'].get()); self.style_widgets[p]['scale'].set(v); self.style_params[p] = v; self.refresh_plots()
        except: pass

    def on_tick_slider(self, p, v):
        if not self.ui_ready: return
        val = float(v); self.style_params[p] = val
        self.tick_widgets[p]['entry'].delete(0, tk.END); self.tick_widgets[p]['entry'].insert(0, f"{val:.3f}"); self.refresh_plots()

    def on_tick_entry(self, p):
        try: v = float(self.tick_widgets[p]['entry'].get()); self.tick_widgets[p]['scale'].set(v); self.style_params[p] = v; self.refresh_plots()
        except: pass

    def sync_time_scale(self, e):
        if not self.ui_ready: return
        for a in ['min','max']: getattr(self, f'ent_{a}').delete(0, tk.END); getattr(self, f'ent_{a}').insert(0, f"{getattr(self, f'scale_{a}').get():.3f}")
        self.refresh_plots()

    def sync_time_entry(self, e):
        try:
            # 1. Считываем значения, очищаем от пробелов и заменяем запятые на точки
            raw_min = self.ent_min.get().replace(',', '.').strip()
            raw_max = self.ent_max.get().replace(',', '.').strip()
            
            # 2. Преобразуем в числа
            val_min = float(raw_min)
            val_max = float(raw_max)
            
            # 3. Блокируем обновление полей ввода на мгновение, чтобы избежать "прыжков" текста
            # Устанавливаем значения ползунков
            self.scale_min.set(val_min)
            self.scale_max.set(val_max)
            
            # 4. Принудительно вызываем обновление графиков
            self.refresh_plots()
            
            # 5. Убираем фокус с поля ввода (опционально, чтобы подтвердить ввод визуально)
            self.root.focus_set()
            
        except ValueError:
            # Если введено что-то некорректное, просто откатываем текст в полях к значениям ползунков
            self.sync_time_scale(None)

    def on_label_update(self, k, en): self.style_params[k] = en.get(); self.refresh_plots()
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
    def export_style(self):
        path = filedialog.asksaveasfilename(defaultextension=".json", filetypes=[("JSON", "*.json")])
        if path:
            data = self.style_params.copy()
            for k, v in self.font_vars.items(): data[k] = v.get()
            data['show_legend'] = self.legend_var.get()
            with open(path, 'w', encoding='utf-8') as f: json.dump(data, f, indent=4, ensure_ascii=False)
    def import_style(self):
        path = filedialog.askopenfilename(filetypes=[("JSON", "*.json")])
        if path:
            try:
                with open(path, 'r', encoding='utf-8') as f: new_s = json.load(f)
                self.style_params.update(new_s); self.legend_var.set(self.style_params.get('show_legend', True))
                for k, v in self.font_vars.items():
                    if k in self.style_params: v.set(self.style_params[k])
                for p, w in self.style_widgets.items():
                    if p in self.style_params:
                        val = self.style_params[p]; w['scale'].set(val); w['entry'].delete(0, tk.END); w['entry'].insert(0, f"{val:.1f}")
                for p, w in self.tick_widgets.items():
                    if p in self.style_params:
                        val = self.style_params[p]; w['scale'].set(val); w['entry'].delete(0, tk.END); w['entry'].insert(0, f"{val:.3f}")
                for k, en in self.label_entries.items():
                    if k in self.style_params: en.delete(0, tk.END); en.insert(0, self.style_params[k])
                self.refresh_plots()
            except: pass

    def reset_all(self):
        self.style_params = self.default_style.copy()
        self.legend_var.set(True)
        for k, v in self.font_vars.items(): v.set(self.style_params[k])
        for p, w in self.style_widgets.items():
            val = self.style_params[p]
            w['scale'].set(val); w['entry'].delete(0, tk.END); w['entry'].insert(0, f"{val:.1f}")
        for p, w in self.tick_widgets.items():
            val = self.style_params[p]
            w['scale'].set(val); w['entry'].delete(0, tk.END); w['entry'].insert(0, f"{val:.3f}")
        for k, en in self.label_entries.items():
            en.delete(0, tk.END); en.insert(0, self.style_params[k])
        self.refresh_plots()

if __name__ == "__main__":
    root = tk.Tk(); app = ChromatogramApp(root); root.mainloop()