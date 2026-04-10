import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.widgets import RangeSlider, Button, TextBox
import tkinter as tk
from tkinter import filedialog
import os

def load_data(file_path):
    """Чтение данных с учетом особенностей вашего CSV (запятая как разделитель разрядов)"""
    try:
        df = pd.read_csv(file_path, sep=';', comment='#', header=None)
        # Если в строке 4 колонки: Point, Min_int, Min_frac, Counts
        if df.shape[1] == 4:
            time = df[1] + df[2] / 1000
            intensity = df[3]
        else:
            time = df.iloc[:, 1]
            intensity = df.iloc[:, 2]
        return time, intensity
    except Exception as e:
        print(f"Ошибка в файле {file_path}: {e}")
        return None, None

def start_analysis():
    root = tk.Tk()
    root.withdraw()
    file_paths = filedialog.askopenfilenames(title="Выберите CSV файлы", filetypes=[("CSV", "*.csv"), ("TXT", "*.txt")])
    
    if not file_paths:
        return

    fig, ax = plt.subplots(figsize=(12, 8))
    plt.subplots_adjust(bottom=0.3) # Увеличили отступ снизу для всех элементов управления

    min_t_global, max_t_global = float('inf'), float('-inf')

    # Загрузка и отрисовка
    for path in file_paths:
        with open(path, 'r') as f:
            header = f.readline()
            label = header.split("Scan")[0].replace('#"', '').strip() if "EIC" in header else os.path.basename(path)
        
        t, y = load_data(path)
        if t is not None:
            ax.plot(t, y, label=label, lw=1.5)
            min_t_global = min(min_t_global, t.min())
            max_t_global = max(max_t_global, t.max())

    ax.set_xlabel('Время (мин)')
    ax.set_ylabel('Интенсивность (Counts)')
    ax.set_title('EIC Chromatograms')
    ax.legend(loc='upper right')
    ax.grid(True, alpha=0.3)

    # --- Виджеты ---
    
    # 1. Ползунок диапазона
    ax_slider = plt.axes([0.2, 0.15, 0.6, 0.03])
    slider = RangeSlider(ax_slider, 'Диапазон', min_t_global, max_t_global, valinit=(min_t_global, max_t_global))

    # 2. Поля для ввода чисел (Левая и Правая границы)
    ax_box_min = plt.axes([0.2, 0.08, 0.1, 0.04])
    ax_box_max = plt.axes([0.7, 0.08, 0.1, 0.04])
    text_min = TextBox(ax_box_min, 'Min: ', initial=f"{min_t_global:.2f}")
    text_max = TextBox(ax_box_max, 'Max: ', initial=f"{max_t_global:.2f}")

    # Функция обновления графика
    def update_plot(val_min, val_max):
        ax.set_xlim(val_min, val_max)
        fig.canvas.draw_idle()

    # Обработка ползунка
    def on_slider_change(val):
        text_min.set_val(f"{val[0]:.2f}")
        text_max.set_val(f"{val[1]:.2f}")
        update_plot(val[0], val[1])

    # Обработка ввода в текстовые поля
    def on_text_submit(text):
        try:
            new_min = float(text_min.text)
            new_max = float(text_max.text)
            if new_min < new_max:
                slider.set_val((new_min, new_max))
                update_plot(new_min, new_max)
        except ValueError:
            pass

    slider.on_changed(on_slider_change)
    text_min.on_submit(on_text_submit)
    text_max.on_submit(on_text_submit)

    # 3. Кнопка сохранения
    ax_button = plt.axes([0.45, 0.02, 0.1, 0.05])
    btn_save = Button(ax_button, 'Save SVG', color='lightgray', hovercolor='0.95')

    def save_svg(event):
        # Скрываем виджеты перед сохранением
        ax_slider.set_visible(False)
        ax_box_min.set_visible(False)
        ax_box_max.set_visible(False)
        ax_button.set_visible(False)
        
        save_path = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        if save_path:
            fig.savefig(save_path, format='svg', bbox_inches='tight')
            print(f"Файл сохранен: {save_path}")
        
        # Возвращаем виджеты
        ax_slider.set_visible(True)
        ax_box_min.set_visible(True)
        ax_box_max.set_visible(True)
        ax_button.set_visible(True)
        fig.canvas.draw()

    btn_save.on_clicked(save_svg)

    plt.show()

if __name__ == "__main__":
    start_analysis()
