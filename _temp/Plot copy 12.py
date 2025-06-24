import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
import dash
from dash import dcc, html, Input, Output, State, no_update
import numpy as np
from scipy.interpolate import griddata

# --- 1. Загрузка и подготовка данных ---
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


# --- 2. Функция для создания начальной фигуры ---
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

# --- 3. Создание Dash приложения ---
app = dash.Dash(__name__)
server = app.server

app.layout = html.Div([
    html.H1("Интерактивный анализ пересечения поверхностей"),
    dcc.Graph(id='main-graph', figure=create_initial_figure(), style={'height': '80vh'}),
    html.Button('Найти пересечение видимых поверхностей', id='intersect-button', n_clicks=0, style={'marginTop': '10px'})
])

# --- 4. Callback для расчета и отображения пересечения ---
@app.callback(
    Output('main-graph', 'figure'),
    Input('intersect-button', 'n_clicks'),
    State('main-graph', 'figure'),
    prevent_initial_call=True
)
def find_and_draw_intersection(n_clicks, fig_dict):
    fig = go.Figure(fig_dict)

    visible_surfaces_data = []
    for trace in fig.data:
        # ИСПРАВЛЕНО: Обращаемся к свойствам объекта напрямую (trace.meta, trace.visible)
        # а не через .get(), как у словаря.
        if hasattr(trace, 'meta') and trace.meta.get('type') == 'surface' and trace.visible in [True, None]:
            visible_surfaces_data.append({
                'x': np.array(trace.x),
                'y': np.array(trace.y),
                'z': np.array(trace.z),
                'acid': trace.meta['acid']
            })

    if len(visible_surfaces_data) < 2:
        print("Для поиска пересечения необходимо как минимум 2 видимые поверхности.")
        return no_update

    print(f"Найдено {len(visible_surfaces_data)} видимых поверхностей. Идет расчет...")

    min_y, max_y = -np.inf, np.inf
    min_z, max_z = -np.inf, np.inf
    for data in visible_surfaces_data:
        min_y = max(min_y, data['y'].min())
        max_y = min(max_y, data['y'].max())
        min_z = max(min_z, data['z'].min())
        max_z = min(max_z, data['z'].max())

    if min_y >= max_y or min_z >= max_z:
        print("Область пересечения поверхностей по осям Y и Z пуста.")
        return no_update

    grid_y, grid_z = np.mgrid[min_y:max_y:100j, min_z:max_z:100j]

    interpolated_x_values = []
    for data in visible_surfaces_data:
        points = np.vstack((data['y'], data['z'])).T
        values = data['x']
        grid_x = griddata(points, values, (grid_y, grid_z), method='linear')
        interpolated_x_values.append(grid_x)

    stacked_x = np.stack(interpolated_x_values, axis=0)
    std_dev = np.nanstd(stacked_x, axis=0)
    
    INTERSECTION_THRESHOLD = 0.1
    intersection_mask = std_dev < INTERSECTION_THRESHOLD

    mean_x = np.nanmean(stacked_x, axis=0)
    intersect_x = mean_x[intersection_mask]
    intersect_y = grid_y[intersection_mask]
    intersect_z = grid_z[intersection_mask]

    fig.data = [trace for trace in fig.data if trace.name != 'Intersection']

    if len(intersect_x) > 0:
        print(f"Найдено {len(intersect_x)} точек пересечения.")
        sort_indices = np.argsort(intersect_y)
        intersect_x, intersect_y, intersect_z = intersect_x[sort_indices], intersect_y[sort_indices], intersect_z[sort_indices]

        fig.add_trace(go.Scatter3d(
            x=intersect_x, y=intersect_y, z=intersect_z,
            mode='lines',
            line=dict(color='red', width=10),
            name='Intersection'
        ))
    else:
        print("Точки пересечения в пределах заданной точности не найдены.")

    return fig

# --- 5. Запуск сервера ---
if __name__ == '__main__':
    app.run(debug=True)