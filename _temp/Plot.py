import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
import dash
from dash import dcc, html, Input, Output, State, no_update
import numpy as np
from scipy.interpolate import LinearNDInterpolator
import matplotlib.pyplot as plt
from itertools import combinations

# --- 1. Загрузка и подготовка данных (без изменений) ---
try:
    df = pd.read_csv('_temp/source.csv')
    df_cleaned = df.dropna(subset=['EquivalentChainLength', 'OnsetTemperature', 'TemperatureStep', 'FattyAcid'])
    df_filtered = df_cleaned[df_cleaned['EquivalentChainLength'] % 1 != 0].copy()
    df_plot = df_filtered.copy()
except FileNotFoundError:
    print("Внимание: Файл '_temp/source.csv' не найден. Для демонстрации созданы случайные данные.")
    data = {
        'FattyAcid': ['AcidA']*20 + ['AcidB']*20 + ['AcidC']*20,
        'OnsetTemperature': list(np.linspace(20, 40, 20)) * 3,
        'TemperatureStep': list(np.linspace(1, 5, 20)) * 3,
        'EquivalentChainLength': np.concatenate([
            25 - 0.1*np.arange(20) - 0.01*np.linspace(1, 5, 20)**2 + np.random.rand(20)*0.2,
            25 - 0.2*np.arange(20) + np.random.rand(20)*0.2,
            24.5 - 0.15*np.arange(20) + 0.02*np.linspace(1, 5, 20)**2 + np.random.rand(20)*0.2
        ])
    }
    df_plot = pd.DataFrame(data)


# --- 2. Функция для создания начальной фигуры (без изменений) ---
def create_initial_figure():
    fig = go.Figure()
    unique_fatty_acids = df_plot['FattyAcid'].unique()
    colors = px.colors.qualitative.Plotly
    surface_trace_indices = []

    for i, acid in enumerate(unique_fatty_acids):
        acid_df = df_plot[df_plot['FattyAcid'] == acid]
        acid_color = colors[i % len(colors)]

        try:
            grid_df = acid_df.pivot_table(
                index='OnsetTemperature', columns='TemperatureStep', values='EquivalentChainLength'
            )
            y_grid_vals, z_grid_vals = grid_df.index.values, grid_df.columns.values
            all_x, all_y, all_z, vertex_map = [], [], [], {}
            v_idx = 0
            for r_idx, y_val in enumerate(y_grid_vals):
                for c_idx, z_val in enumerate(z_grid_vals):
                    x_val = grid_df.loc[y_val, z_val]
                    if pd.notna(x_val):
                        all_x.append(x_val); all_y.append(y_val); all_z.append(z_val)
                        vertex_map[(r_idx, c_idx)] = v_idx
                        v_idx += 1

            faces_i, faces_j, faces_k = [], [], []
            for r_idx in range(len(y_grid_vals) - 1):
                for c_idx in range(len(z_grid_vals) - 1):
                    p1, p2, p3, p4 = (r_idx, c_idx), (r_idx, c_idx + 1), (r_idx + 1, c_idx + 1), (r_idx + 1, c_idx)
                    if all(p in vertex_map for p in [p1, p2, p3, p4]):
                        v1, v2, v3, v4 = vertex_map[p1], vertex_map[p2], vertex_map[p3], vertex_map[p4]
                        faces_i.extend([v1, v1]); faces_j.extend([v2, v3]); faces_k.extend([v3, v4])

            if faces_i:
                fig.add_trace(go.Mesh3d(
                    x=all_x, y=all_y, z=all_z,
                    i=faces_i, j=faces_j, k=faces_k,
                    color=acid_color, opacity=0.5,
                    legendgroup=acid, name=acid, showlegend=False, hoverinfo='none',
                    meta={'type': 'surface', 'acid': acid}
                ))
                surface_trace_indices.append(len(fig.data) - 1)

        except Exception as e:
            print(f"Не удалось создать поверхность для {acid}: {e}")

        for step in acid_df['TemperatureStep'].unique():
            step_df = acid_df[acid_df['TemperatureStep'] == step]
            fig.add_trace(go.Scatter3d(
                x=step_df['EquivalentChainLength'], y=step_df['OnsetTemperature'], z=step_df['TemperatureStep'],
                mode='markers',
                marker=dict(size=5, color=acid_color, symbol='circle', line=dict(color='black', width=1)),
                legendgroup=acid, name=acid,
                showlegend=bool(step == acid_df['TemperatureStep'].unique()[0]),
                hovertext=step_df['FattyAcid'],
                hovertemplate=
                    '<b>%{hovertext}</b><br><br>' +
                    'Equivalent Chain Length: %{x:.2f}<br>' +
                    'Onset Temperature: %{y:.2f}<br>' +
                    'Temperature Step: %{z:.2f}<extra></extra>',
                meta={'type': 'scatter', 'acid': acid}
            ))

    fig.update_layout(
        title='3D-график жирных кислот с поверхностью по сетке',
        scene=dict(
            xaxis_title='Equivalent Chain Length',
            yaxis_title='Onset Temperature',
            zaxis_title='Temperature Step'
        ),
        legend=dict(
            title=dict(text='Fatty Acid (нажмите)'),
            x=0, y=1, xanchor="left", yanchor="top",
            bgcolor="rgba(255, 255, 255, 0.7)", bordercolor="Black", borderwidth=1
        ),
        margin=dict(l=0, r=0, b=0, t=40),
        updatemenus=[
            dict(
                type="buttons", direction="right", active=0,
                buttons=[
                    dict(label="Показать поверхности", method="restyle", args=[{"visible": True}, surface_trace_indices]),
                    dict(label="Скрыть поверхности", method="restyle", args=[{"visible": False}, surface_trace_indices]),
                ],
                pad={"r": 10, "t": 10}, showactive=True,
                x=0.01, xanchor="left", y=1.1, yanchor="top"
            )
        ]
    )
    return fig

# --- 3. Создание Dash приложения (без изменений) ---
app = dash.Dash(__name__)
server = app.server

app.layout = html.Div([
    html.H1("Интерактивный анализ пересечения поверхностей"),
    dcc.Graph(id='main-graph', figure=create_initial_figure(), style={'height': '80vh'}),
    html.Button('Найти пересечение видимых поверхностей', id='intersect-button', n_clicks=0, style={'marginTop': '10px'})
])

# --- 4. Callback с НОВЫМ алгоритмом поиска пересечения ---
@app.callback(
    Output('main-graph', 'figure'),
    Input('intersect-button', 'n_clicks'),
    State('main-graph', 'figure'),
    prevent_initial_call=True
)
def find_and_draw_intersection(n_clicks, fig_dict):
    fig = go.Figure(fig_dict)

    # Сначала удаляем старые линии пересечения
    fig.data = [trace for trace in fig.data if trace.name != 'Intersection']

    # --- Шаг А: Найти видимые поверхности ---
    visible_surfaces = []
    for trace in fig.data:
        if hasattr(trace, 'meta') and trace.meta.get('type') == 'surface' and trace.visible in [True, None]:
            visible_surfaces.append(trace)

    if len(visible_surfaces) < 2:
        print("Для поиска пересечения необходимо как минимум 2 видимые поверхности.")
        return fig # Возвращаем фигуру с удаленными старыми линиями

    print(f"Найдено {len(visible_surfaces)} видимых поверхностей. Идет расчет пересечений...")

    # --- Шаг Б: Итерируемся по всем парам видимых поверхностей ---
    total_segments_found = 0
    for surf_A, surf_B in combinations(visible_surfaces, 2):
        print(f"  - Ищем пересечение между '{surf_A.meta['acid']}' и '{surf_B.meta['acid']}'")

        # --- Шаг В: Создаем интерполяторы для каждой поверхности x = f(y, z) ---
        points_A = np.vstack((surf_A.y, surf_A.z)).T
        values_A = surf_A.x
        interp_A = LinearNDInterpolator(points_A, values_A)

        points_B = np.vstack((surf_B.y, surf_B.z)).T
        values_B = surf_B.x
        interp_B = LinearNDInterpolator(points_B, values_B)

        # --- Шаг Г: Определяем общую область для анализа ---
        min_y = max(points_A[:, 0].min(), points_B[:, 0].min())
        max_y = min(points_A[:, 0].max(), points_B[:, 0].max())
        min_z = max(points_A[:, 1].min(), points_B[:, 1].min())
        max_z = min(points_A[:, 1].max(), points_B[:, 1].max())

        if min_y >= max_y or min_z >= max_z:
            print("    - Поверхности не пересекаются в плоскости Y-Z.")
            continue

        # --- Шаг Д: Создаем сетку и находим разницу высот (X) ---
        grid_y_1d = np.linspace(min_y, max_y, 100)
        grid_z_1d = np.linspace(min_z, max_z, 100)
        grid_yy, grid_zz = np.meshgrid(grid_y_1d, grid_z_1d)

        x_A_on_grid = interp_A((grid_yy, grid_zz))
        x_B_on_grid = interp_B((grid_yy, grid_zz))

        # Разница высот. Ищем где она равна 0
        diff_grid = x_A_on_grid - x_B_on_grid

        # --- Шаг Е: Используем Matplotlib для поиска контура, где разница = 0 ---
        # plt.contour не рисует график, а вычисляет координаты линий
        cs = plt.contour(grid_y_1d, grid_z_1d, diff_grid, levels=[0])
        
        # allsegs[0] содержит все найденные отрезки для уровня 0
        intersection_segments = cs.allsegs[0]
        
        if not any(s.any() for s in intersection_segments):
            print("    - Линия пересечения не найдена.")
            continue

        # --- Шаг Ж: Добавляем найденные линии на график ---
        for seg in intersection_segments:
            if len(seg) < 2: continue # Нужна хотя бы линия из 2 точек
            
            # seg содержит координаты (y, z)
            seg_y, seg_z = seg[:, 0], seg[:, 1]
            # Находим для них координату x (можно использовать любой интерполятор)
            seg_x = interp_A(seg_y, seg_z)

            # Добавляем разрыв в линии (None), чтобы Plotly не соединял отдельные сегменты
            fig.add_trace(go.Scatter3d(
                x=seg_x, y=seg_y, z=seg_z,
                mode='lines',
                line=dict(color='red', width=10),
                name='Intersection',
                showlegend=False
            ))
            total_segments_found += 1
    
    plt.close('all') # Закрываем фигуры matplotlib, чтобы они не отображались
    print(f"Расчет завершен. Найдено и отрисовано {total_segments_found} сегментов пересечения.")
    return fig

# --- 5. Запуск сервера (без изменений) ---
if __name__ == '__main__':
    app.run(debug=True)