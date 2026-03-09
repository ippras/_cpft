import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from scipy.interpolate import griddata
from matplotlib.widgets import Button

# 1. Данные из таблицы
data = [
    [60.0, 1.0, 0.991303],
    [60.0, 2.0, 0.998611],
    [60.0, 3.0, 0.998754],
    [60.0, 4.0, 0.998926],
    [60.0, 5.0, 0.998391],
    [60.0, 6.0, 0.997165],
    [60.0, 7.0, 0.99626],
    [60.0, 8.0, 0.995403],
    [60.0, 9.0, 0.994563],
    [60.0, 10.0, 0.993432],
    [70.0, 1.0, 0.990623],
    [70.0, 2.0, 0.995743],
    [70.0, 3.0, 0.9987],
    [70.0, 4.0, 0.999688],
    [70.0, 5.0, 0.99822],
    [70.0, 6.0, 0.996917],
    [70.0, 7.0, 0.996027],
    [70.0, 8.0, 0.995193],
    [70.0, 9.0, 0.993941],
    [70.0, 10.0, 0.993011],
    [80.0, 1.0, 0.989666],
    [80.0, 2.0, 0.99534],
    [80.0, 3.0, 0.998523],
    [80.0, 4.0, 0.99979],
    [80.0, 5.0, 0.998138],
    [80.0, 6.0, 0.996813],
    [80.0, 7.0, 0.995759],
    [80.0, 8.0, 0.994833],
    [80.0, 9.0, 0.993496],
    [80.0, 10.0, 0.992627],
    [90.0, 1.0, 0.988617],
    [90.0, 2.0, 0.99488],
    [90.0, 3.0, 0.998334],
    [90.0, 4.0, 0.999579],
    [90.0, 5.0, 0.997877],
    [90.0, 6.0, 0.996365],
    [90.0, 7.0, 0.995355],
    [90.0, 8.0, 0.994352],
    [90.0, 9.0, 0.993126],
    [90.0, 10.0, 0.991975],
    [100.0, 1.0, 0.98761],
    [100.0, 2.0, 0.994559],
    [100.0, 3.0, 0.99832],
    [100.0, 4.0, 0.999591],
    [100.0, 5.0, 0.997751],
    [100.0, 6.0, 0.996266],
    [100.0, 7.0, 0.995188],
    [100.0, 8.0, 0.993815],
    [100.0, 9.0, 0.992435],
    [100.0, 10.0, 0.991329],
    [110.0, 1.0, 0.986245],
    [110.0, 2.0, 0.99407],
    [110.0, 3.0, 0.998198],
    [110.0, 4.0, 0.999346],
    [110.0, 5.0, 0.997385],
    [110.0, 6.0, 0.995837],
    [110.0, 7.0, 0.994366],
    [110.0, 8.0, 0.993229],
    [110.0, 9.0, 0.991761],
    [110.0, 10.0, 0.990567],
    [120.0, 1.0, 0.984922],
    [120.0, 2.0, 0.993758],
    [120.0, 3.0, 0.998321],
    [120.0, 4.0, 0.999667],
    [120.0, 5.0, 0.999028],
    [120.0, 6.0, 0.995213],
    [120.0, 7.0, 0.994016],
    [120.0, 8.0, 0.992526],
    [120.0, 9.0, 0.99086],
    [120.0, 10.0, 0.989522],
    [130.0, 1.0, 0.9834],
    [130.0, 2.0, 0.993587],
    [130.0, 3.0, 0.998693],
    [130.0, 4.0, 0.998724],
    [130.0, 5.0, 0.996271],
    [130.0, 6.0, 0.994326],
    [130.0, 7.0, 0.992603],
    [130.0, 8.0, 0.990992],
    [130.0, 9.0, 0.989562],
    [130.0, 10.0, 0.987776],
    [140.0, 1.0, 0.981788],
    [140.0, 2.0, 0.993607],
    [140.0, 3.0, 0.999737],
    [140.0, 4.0, 0.997809],
    [140.0, 5.0, 0.995077],
    [140.0, 6.0, 0.993027],
    [140.0, 7.0, 0.991426],
    [140.0, 8.0, 0.989743],
    [140.0, 9.0, 0.987843],
    [140.0, 10.0, 0.986032],
    [150.0, 1.0, 0.982911],
    [150.0, 2.0, 0.994818],
    [150.0, 3.0, 0.999692],
    [150.0, 4.0, 0.995914],
    [150.0, 5.0, 0.993319],
    [150.0, 6.0, 0.990866],
    [150.0, 7.0, 0.989121],
    [150.0, 8.0, 0.987288],
    [150.0, 9.0, 0.984799],
    [150.0, 10.0, 0.983184]
]

df = pd.DataFrame(data, columns=['T0', 'dT', 'alpha'])

# 2. Интерполяция поверхности alpha
t0_fine = np.linspace(df['T0'].min(), df['T0'].max(), 100)
dt_fine = np.linspace(df['dT'].min(), df['dT'].max(), 100)
T0_mesh, DT_mesh = np.meshgrid(t0_fine, dt_fine)

Z_alpha = griddata((df['T0'], df['dT']), df['alpha'], (T0_mesh, DT_mesh), method='cubic')

# 3. Поиск линии пересечения (где alpha = 1.0)
fig_temp, ax_temp = plt.subplots()
contour = ax_temp.contour(T0_mesh, DT_mesh, Z_alpha, levels=[1.0])
plt.close(fig_temp)

inter_pts = []
paths = contour.get_paths()
for path in paths:
    v = path.vertices
    # Для каждой точки на контуре alpha=1.0
    for x, y in zip(v[:,0], v[:,1]):
        inter_pts.append([x, y, 1.0])

df_inter = pd.DataFrame(inter_pts, columns=['T0', 'dT', 'alpha'])
print("\nТАБЛИЦА ТОЧЕК ПЕРЕСЕЧЕНИЯ (alpha = 1.0):")
print(df_inter.to_string(index=False))

# 4. Визуализация
fig = plt.figure(figsize=(12, 8))
ax = fig.add_subplot(111, projection='3d')
plt.subplots_adjust(bottom=0.2)

# Поверхность alpha
surf = ax.plot_surface(T0_mesh, DT_mesh, Z_alpha, alpha=0.4, cmap='viridis')

# Экспериментальные точки
sc_pts = ax.scatter(df['T0'], df['dT'], df['alpha'], c='blue', s=30, edgecolors='black', label='Эксп. данные')

# Линия пересечения (alpha = 1)
if inter_pts:
    pts_arr = np.array(inter_pts)
    ax.plot(pts_arr[:,0], pts_arr[:,1], pts_arr[:,2], color='red', linewidth=4, zorder=20)
    sc_inter = ax.scatter(pts_arr[::5,0], pts_arr[::5,1], pts_arr[::5,2], 
                          color='red', s=50, edgecolors='white', label='Линия alpha=1.0')

# --- ПОДСКАЗКА (Figure Pixels - не прилипает к стенкам) ---
annot = ax.annotate("", xy=(0,0), xytext=(15, 15), textcoords="offset points",
                    bbox=dict(boxstyle="round", fc="white", ec="gray", alpha=0.9),
                    arrowprops=dict(arrowstyle="->", color='gray'),
                    xycoords='figure pixels')
annot.set_visible(False)

def hover(event):
    if event.inaxes == ax:
        # Проверяем точки данных и точки пересечения
        targets = [(sc_pts, "Данные")]
        if 'sc_inter' in locals(): targets.append((sc_inter, "Alpha=1.0"))
            
        for sc, label in targets:
            cont, ind = sc.contains(event)
            if cont:
                pos = sc._offsets3d
                idx = ind["ind"][0]
                x, y, z = pos[0][idx], pos[1][idx], pos[2][idx]
                annot.set_text(f"Тип: {label}\nT0: {x:.2f}\ndT: {y:.2f}\nAlpha: {z:.4f}")
                annot.xy = (event.x, event.y)
                annot.set_visible(True)
                fig.canvas.draw_idle()
                return
    if annot.get_visible():
        annot.set_visible(False)
        fig.canvas.draw_idle()

fig.canvas.mpl_connect("motion_notify_event", hover)

# Кнопки управления
def set_view(elev, azim, title):
    ax.view_init(elev=elev, azim=azim)
    ax.set_title(title)
    plt.draw()

ax_res = plt.axes([0.05, 0.05, 0.18, 0.05]); btn_res = Button(ax_res, '3D Сброс')
ax_top = plt.axes([0.25, 0.05, 0.18, 0.05]); btn_top = Button(ax_top, 'T0 / dT (Top)')
ax_s1  = plt.axes([0.45, 0.05, 0.18, 0.05]); btn_s1  = Button(ax_s1, 'T0 / Alpha')
ax_s2  = plt.axes([0.65, 0.05, 0.18, 0.05]); btn_s2  = Button(ax_s2, 'dT / Alpha')

btn_res.on_clicked(lambda x: set_view(20, -45, "Объемный вид"))
btn_top.on_clicked(lambda x: set_view(90, -90, "Вид сверху (Зона коэлюции)"))
btn_s1.on_clicked(lambda x: set_view(0, -90, "Зависимость Alpha от T0"))
btn_s2.on_clicked(lambda x: set_view(0, 0, "Зависимость Alpha от dT"))

ax.set_xlabel('T0 (Начальная темп.)')
ax.set_ylabel('dT (Скорость нагрева)')
ax.set_zlabel('Alpha (Селективность)')
set_view(20, -45, "Поверхность селективности Alpha")

plt.show()