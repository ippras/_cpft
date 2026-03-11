import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.widgets import RangeSlider

def plot_with_sliders(csv_file):
    # 1. Загрузка и подготовка данных
    # Читаем CSV, учитывая, что X разделен запятой (6,591 -> две колонки)
    try:
        df = pd.read_csv(csv_file, comment='#', header=None, 
                         names=['point', 'x_int', 'x_frac', 'y'])
    except Exception as e:
        print(f"Ошибка при чтении файла: {e}")
        return

    # Собираем X обратно в число (6 + 591/1000 = 6.591)
    df['x'] = df['x_int'] + df['x_frac'] / 1000.0

    # 2. Создание графического интерфейса
    fig, ax = plt.subplots(figsize=(12, 7))
    plt.subplots_adjust(bottom=0.25) # Оставляем место внизу для ползунка

    # Рисуем основной график
    line, = ax.plot(df['x'], df['y'], color='#1f77b4', lw=1)
    ax.set_xlabel('Time (Minutes)')
    ax.set_ylabel('Counts')
    ax.set_title('TIC Scan Chromatogram')
    ax.grid(True, linestyle='--', alpha=0.5)

    # 3. Настройка ползунка диапазона
    # Создаем оси для слайдера [left, bottom, width, height]
    ax_slider = plt.axes([0.15, 0.1, 0.7, 0.03])
    
    slider = RangeSlider(
        ax_slider, "Range X", 
        df['x'].min(), df['x'].max(), 
        valinit=(df['x'].min(), df['x'].max())
    )

    # Функция обновления масштаба
    def update(val):
        # val[0] - левая граница, val[1] - правая
        ax.set_xlim(val[0], val[1])
        
        # Автоматическое масштабирование Y под выбранный диапазон
        visible_y = df[(df['x'] >= val[0]) & (df['x'] <= val[1])]['y']
        if not visible_y.empty:
            ax.set_ylim(visible_y.min() * 0.95, visible_y.max() * 1.05)
        
        fig.canvas.draw_idle()

    slider.on_changed(update)

    print("Инструкция:")
    print("1. Используйте ползунки внизу для выбора диапазона.")
    print("2. Нажмите на иконку дискеты в окне графика.")
    print("3. В поле 'Тип файла' выберите .svg")

    plt.show()

# Замените 'data.csv' на имя вашего файла
plot_with_sliders('source/80C_1ml-min_4C-min_1.CSV')