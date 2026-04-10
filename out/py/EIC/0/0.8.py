import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.widgets import RangeSlider, Button, TextBox, RadioButtons
import tkinter as tk
from tkinter import filedialog
import os
import numpy as np

# Хранилище данных
plots_registry = []
all_data_bounds = [float('inf'), float('-inf')]

def load_data(file_path):
    """Чтение данных с разделителем ';' и десятичной запятой"""
    try:
        df = pd.read_csv(file_path, sep=';', decimal=',', comment='#', header=None)
        if df.shape[1] >= 3:
            time = pd.to_numeric(df.iloc[:, 1], errors='coerce')
            intensity = pd.to_numeric(df.iloc[:, 2], errors='coerce')
            mask = time.notna() & intensity.notna()
            return time[mask].values, intensity[mask].values
        return None, None
    except Exception as e:
        print(f"Ошибка в файле {file_path}: {e}")
        return None, None

def start_analysis():
    fig, ax = plt.subplots(figsize=(12, 8))
    plt.subplots_adjust(bottom=0.35, left=0.1, right=0.82)

    ax.set_xlabel('Время (мин)')
    ax.set_ylabel('Интенсивность (Counts)')
    ax.grid(True, alpha=0.3)

    # --- Виджеты ---
    ax_slider = plt.axes([0.2, 0.22, 0.5, 0.03])
    slider = RangeSlider(ax_slider, 'Диапазон', 0, 1, valinit=(0, 1))

    ax_box_min = plt.axes([0.2, 0.15, 0.1, 0.04])
    ax_box_max = plt.axes([0.6, 0.15, 0.1, 0.04])
    text_min = TextBox(ax_box_min, 'Min: ', initial="0")
    text_max = TextBox(ax_box_max, 'Max: ', initial="1")

    ax_radio = plt.axes([0.84, 0.5, 0.15, 0.12], facecolor='#f0f0f0')
    radio = RadioButtons(ax_radio, ('Абсолютный', 'Нормированный'))

    def get_current_mode():
        return radio.value_selected

    def fit_y_to_visible(event=None):
        """Умный автомасштаб Y: подстраивается под максимум в видимом окне X"""
        x_min, x_max = ax.get_xlim()
        max_y_visible = 0
        
        for item in plots_registry:
            # Берем текущие Y-данные (они могут быть нормированы или нет)
            y_data = item['line'].get_ydata()
            t_data = item['t']
            
            # Находим данные, попадающие в текущий диапазон X
            mask = (t_data >= x_min) & (t_data <= x_max)
            visible_y = y_data[mask]
            
            if len(visible_y) > 0:
                max_y_visible = max(max_y_visible, np.max(visible_y))
        
        if max_y_visible > 0:
            ax.set_ylim(-max_y_visible * 0.05, max_y_visible * 1.1)
        fig.canvas.draw_idle()

    def update_mode(label=None):
        """Переключение между Counts и %"""
        mode = get_current_mode()
        for item in plots_registry:
            if mode == 'Нормированный':
                item['line'].set_ydata((item['y'] / item['max_y']) * 100)
                ax.set_ylabel('Относительная интенсивность (%)')
            else:
                item['line'].set_ydata(item['y'])
                ax.set_ylabel('Интенсивность (Counts)')
        fit_y_to_visible()

    radio.on_clicked(update_mode)

    def add_files(event):
        root = tk.Tk(); root.withdraw()
        file_paths = filedialog.askopenfilenames(filetypes=[("Data", "*.csv *.txt"), ("All", "*.*")])
        root.destroy()
        if not file_paths: return

        for path in file_paths:
            with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                header = f.readline()
                label = header.split("Scan")[0].replace('#"', '').strip() if "EIC" in header else os.path.basename(path)
            
            t, y = load_data(path)
            if t is not None and len(t) > 0:
                line, = ax.plot(t, y, label=label, lw=1.2)
                plots_registry.append({'t': t, 'y': y, 'max_y': np.max(y), 'line': line})
                all_data_bounds[0] = min(all_data_bounds[0], np.min(t))
                all_data_bounds[1] = max(all_data_bounds[1], np.max(t))

        if plots_registry:
            slider.valmin = all_data_bounds[0]
            slider.valmax = all_data_bounds[1]
            ax_slider.set_xlim(slider.valmin, slider.valmax)
            ax.legend(loc='upper right', fontsize='x-small')
            # При первой загрузке сбрасываем вид на полный
            reset_view(None)

    def reset_view(event):
        """Сброс масштаба на полный диапазон"""
        if not plots_registry: return
        slider.set_val((all_data_bounds[0], all_data_bounds[1]))
        update_mode()

    # --- Кнопки управления ---
    ax_add = plt.axes([0.15, 0.02, 0.12, 0.05])
    btn_add = Button(ax_add, '➕ Добавить', color='#e1ffc7')
    btn_add.on_clicked(add_files)

    ax_fit = plt.axes([0.28, 0.02, 0.12, 0.05])
    btn_fit = Button(ax_fit, '🔍 Подогнать Y', color='#fff3c7')
    btn_fit.on_clicked(fit_y_to_visible)

    ax_reset = plt.axes([0.41, 0.02, 0.12, 0.05])
    btn_reset = Button(ax_reset, '🔄 Сброс вида', color='#f0f0f0')
    btn_reset.on_clicked(reset_view)

    ax_clear = plt.axes([0.54, 0.02, 0.1, 0.05])
    btn_clear = Button(ax_clear, '🗑️ Очистить', color='#ffc7c7')
    
    def clear_all(event):
        for item in plots_registry: item['line'].remove()
        plots_registry.clear()
        all_data_bounds[0], all_data_bounds[1] = float('inf'), float('-inf')
        ax.legend_ = None
        ax.set_xlim(0, 1); ax.set_ylim(0, 1)
        fig.canvas.draw_idle()
    btn_clear.on_clicked(clear_all)

    ax_save = plt.axes([0.75, 0.02, 0.12, 0.05])
    btn_save = Button(ax_save, '💾 Save SVG', color='#c7e9ff')

    # Логика слайдера
    def on_slider_change(val):
        ax.set_xlim(val[0], val[1])
        text_min.set_val(f"{val[0]:.2f}")
        text_max.set_val(f"{val[1]:.2f}")
        # Опционально: можно включить авто-Y при каждом движении слайдера, 
        # но это может "дергать" график, поэтому лучше оставить кнопку "Подогнать Y"
        fig.canvas.draw_idle()

    slider.on_changed(on_slider_change)

    def on_text_submit(text):
        try:
            v_min = float(text_min.text.replace(',', '.'))
            v_max = float(text_max.text.replace(',', '.'))
            slider.set_val((v_min, v_max))
        except: pass
    text_min.on_submit(on_text_submit)
    text_max.on_submit(on_text_submit)

    def save_svg(event):
        widgets = [ax_slider, ax_box_min, ax_box_max, ax_add, ax_fit, ax_reset, ax_clear, ax_save, ax_radio]
        for w in widgets: w.set_visible(False)
        root = tk.Tk(); root.withdraw()
        save_path = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        root.destroy()
        if save_path: fig.savefig(save_path, format='svg', bbox_inches='tight')
        for w in widgets: w.set_visible(True)
        fig.canvas.draw()
    btn_save.on_clicked(save_svg)

    add_files(None)
    plt.show()

if __name__ == "__main__":
    start_analysis()