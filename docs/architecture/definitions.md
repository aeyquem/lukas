# Definitions

## Frontend

- Bases: these are the smallest unit in the system, they have no specific responsibility but are used to build larger components across the application. right now they mimic the primevue components 1:1 but we can add more if we need to.
- Blocks: these are larger components that are composed of base components and are used to build the application's UI. they have one specific purpose (like the period selector) and are composed of base components to achieve that purpose.
- Pages: these are the top-most-level components and are used to move the user from one part to another. for example the home page and the app page. these are usually meant to change the full viewport and not just a part of it.
- Views: these are another top-level components that are used to display information to the user. they are composed of blocks and layouts to achieve the desired UI. they are usually shown in the main area of a layout and are the content of a page.
- Layouts: these are presentational components that define how a page or view should look in a macro level. they define the position of other elements like a sidebar, header, footer, drawers and can have some behavior to work with them (like hiding the sidebar on small screens)
