import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.widgets import RangeSlider, Button

def plot_clean_line(csv_file):
    # 1. Загрузка данных
    try:
        # Читаем CSV (учитываем запятую в X: 6,591 -> две колонки)
        df = pd.read_csv(csv_file, comment='#', header=None, 
                         names=['point', 'x_int', 'x_frac', 'y'])
        df['x'] = df['x_int'] + df['x_frac'] / 1000.0
    except Exception as e:
        print(f"Ошибка: {e}")
        return

    # 2. Создание окна
    fig, ax = plt.subplots(figsize=(12, 6))
    plt.subplots_adjust(bottom=0.25, left=0.05, right=0.95, top=0.95)

    # Рисуем линию
    line, = ax.plot(df['x'], df['y'], color='black', linewidth=1)

    # --- ОЧИСТКА ГРАФИКА ---
    ax.set_axis_off() # Убираем оси, деления и рамку
    fig.patch.set_visible(False) # Делаем фон прозрачным
    ax.patch.set_visible(False)

    # 3. Добавление ползунка (RangeSlider)
    ax_slider = plt.axes([0.15, 0.1, 0.7, 0.03])
    slider = RangeSlider(
        ax_slider, "Time Range", 
        df['x'].min(), df['x'].max(), 
        valinit=(df['x'].min(), df['x'].max())
    )

    # Обновление диапазона
    def update(val):
        ax.set_xlim(val[0], val[1])
        # Авто-масштаб по высоте для удобства просмотра
        visible_y = df[(df['x'] >= val[0]) & (df['x'] <= val[1])]['y']
        if not visible_y.empty:
            ax.set_ylim(visible_y.min(), visible_y.max())
        fig.canvas.draw_idle()

    slider.on_changed(update)

    # 4. Кнопка сохранения (только линии)
    ax_save = plt.axes([0.85, 0.02, 0.1, 0.04])
    btn_save = Button(ax_save, 'Save SVG')

    def save_svg(event):
        # Сохраняем только область ax (где линия), игнорируя слайдеры и кнопки
        extent = ax.get_window_extent().transformed(fig.dpi_scale_trans.inverted())
        filename = f"line_{slider.val[0]:.2f}-{slider.val[1]:.2f}.svg"
        
        # Сохраняем с прозрачным фоном и без полей
        fig.savefig(filename, format='svg', transparent=True, 
                    bbox_inches=extent, pad_inches=0)
        print(f"Сохранено как: {filename}")

    btn_save.on_clicked(save_svg)

    print("Инструкция:")
    print("1. Выберите диапазон ползунками внизу.")
    print("2. Нажмите кнопку 'Save SVG' в углу экрана.")
    print("3. В файл попадет ТОЛЬКО линия внутри выбранного окна.")

    plt.show()

# Укажите имя вашего файла
plot_clean_line('py/60C_1ml-min_1C-min_2.CSV')