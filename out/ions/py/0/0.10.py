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

    def refresh_plots(val=None):
        """Основная функция перерисовки"""
        if not plots_registry:
            return

        x_min, x_max = slider.val
        mode = radio.value_selected
        
        ax.set_xlim(x_min, x_max)
        text_min.set_val(f"{x_min:.2f}")
        text_max.set_val(f"{x_max:.2f}")

        if mode == 'Нормированный':
            ax.set_ylabel('Относительная интенсивность (%)')
            for item in plots_registry:
                # Находим локальный максимум в окне для каждого файла
                mask = (item['t'] >= x_min) & (item['t'] <= x_max)
                visible_y = item['y_orig'][mask]
                
                if len(visible_y) > 0 and np.max(visible_y) > 0:
                    local_max = np.max(visible_y)
                    # Нормируем данные линии (каждая линия по своему максу в окне)
                    item['line'].set_ydata((item['y_orig'] / local_max) * 100)
                else:
                    item['line'].set_ydata(item['y_orig'] * 0)
            
            ax.set_ylim(-2, 105) # В нормированном режиме шкала всегда 0-100%
        
        else: # АБСОЛЮТНЫЙ РЕЖИМ
            ax.set_ylabel('Интенсивность (Counts)')
            max_y_visible = 0
            for item in plots_registry:
                # Возвращаем исходные данные без нормировки
                item['line'].set_ydata(item['y_orig'])
                
                # Ищем самый высокий пик среди всех файлов в видимом окне для масштаба оси Y
                mask = (item['t'] >= x_min) & (item['t'] <= x_max)
                visible_y = item['y_orig'][mask]
                if len(visible_y) > 0:
                    max_y_visible = max(max_y_visible, np.max(visible_y))
            
            # Подстраиваем высоту шкалы Y под реальные значения Counts в окне
            if max_y_visible > 0:
                ax.set_ylim(-max_y_visible * 0.02, max_y_visible * 1.1)
            else:
                ax.set_ylim(-1, 10)

        fig.canvas.draw_idle()

    # Привязываем события
    slider.on_changed(refresh_plots)
    radio.on_clicked(refresh_plots)

    def on_text_submit(text):
        try:
            v_min = float(text_min.text.replace(',', '.'))
            v_max = float(text_max.text.replace(',', '.'))
            slider.set_val((v_min, v_max))
        except: pass

    text_min.on_submit(on_text_submit)
    text_max.on_submit(on_text_submit)

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
                plots_registry.append({'t': t, 'y_orig': y, 'line': line})
                all_data_bounds[0] = min(all_data_bounds[0], np.min(t))
                all_data_bounds[1] = max(all_data_bounds[1], np.max(t))

        if plots_registry:
            slider.valmin = all_data_bounds[0]
            slider.valmax = all_data_bounds[1]
            ax_slider.set_xlim(slider.valmin, slider.valmax)
            ax.legend(loc='upper right', fontsize='x-small')
            slider.set_val((all_data_bounds[0], all_data_bounds[1]))

    def clear_all(event):
        for item in plots_registry: item['line'].remove()
        plots_registry.clear()
        all_data_bounds[0], all_data_bounds[1] = float('inf'), float('-inf')
        ax.legend_ = None
        ax.set_xlim(0, 1); ax.set_ylim(0, 1)
        fig.canvas.draw_idle()

    # Кнопки
    ax_add = plt.axes([0.15, 0.02, 0.15, 0.05])
    btn_add = Button(ax_add, '➕ Добавить', color='#e1ffc7')
    btn_add.on_clicked(add_files)

    ax_clear = plt.axes([0.32, 0.02, 0.15, 0.05])
    btn_clear = Button(ax_clear, '🗑️ Очистить', color='#ffc7c7')
    btn_clear.on_clicked(clear_all)

    ax_save = plt.axes([0.65, 0.02, 0.15, 0.05])
    btn_save = Button(ax_save, '💾 Save SVG', color='#c7e9ff')

    def save_svg(event):
        widgets = [ax_slider, ax_box_min, ax_box_max, ax_add, ax_clear, ax_save, ax_radio]
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