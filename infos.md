
# Frame

Le moteur autorise la génération de plusieurs images simultanées.
une "Frame" est représentée par un ID (générallement entre 0 et 2 pour 3 images simultannées)

# Resource graphique

Les resources graphiques sont les resources utilisées à bas niveau
par le renderer pour le rendu. (textures / mesh / framebuffer etc...)

Ces resources sont soit dynamiques, soit statiques.

- Les resources statiques peuvent être utilisées simultanément par plusieurs renderers et plusieurs frames.
- Les resources dynamiques sont automatiquement dupliquées pour chaque render_pass et chaque frame. 
Il n'y a pas de lien entre chaque instance.

Une resource dynamique est utilisée avec un DynamiqueResourceID

# Renderer

Un renderer est une description d'une pipeline de rendu à haut niveau d'abstraction.

Un renderer est ensuite compilé puis instancié en **frame_graph**.
La compilation optimise les resources (textures de framebuffer) pour les réutiliser au maximum.
L'instanciation peut viser une surface (fenêtre), ou une texture (render target).

Il n'y a pas d'optimisation inter-frame_graph. On est censé n'avoir qu'un seul
frame_graph dans le cadre d'un jeu packaged (dans la majorité des cas).

Dans le cas contraire, les resources non-statiques sont dynamiquement duppliquées


## A savoir

- Le frame_graph contient le nombre d'images à rendre en parallèle maximum.
- Au moment d'être utilisée pour le rendu, une ressource doit connaitre son renderer. (avec possibilité de pré-compiler pour ce renderer)

## Construction

Le renderer contient un "present node" qui contiendra récursivement toutes ses dépendances.

Chaque node sera instancié en "render_pass"

## Nommages

| High-Level | Low-Level (compiled)              | Backend                               |
|------------|-----------------------------------|---------------------------------------|
| Renderer   | FrameGraph                        | -                                     |
| RenderNode | RenderPass [ RenderPassInstance ] | VkRenderPassInstance < VkRenderPass > |

## Schemas de rendu
```
Engine::engine_loop()
├── Platform::poll_events()
└── for renderer in renderers:
    └── Renderer::new_frame()
        └── for frame_graph in frame_graphs:
            └── Framegraph::execute()
                ├── RenderPass::draw()
                ├── for input in inputs
                │   └── RenderPass::draw()
                ├── RenderPassInstance::bind()
                ├── // draw stuffs...
                └── RenderPassInstance::submit()
```