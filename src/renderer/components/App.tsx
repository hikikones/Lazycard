import * as React from 'react';
import { MemoryRouter } from 'react-router-dom';

import cfg from '../model/Config';

import Layout, { Content } from './Layout';
import Nav from './Nav';
import Routes from './Routes';

const parseTheme = (): string => {
    const theme = cfg.getTheme();
    return theme === "system" ? getSystemTheme() : theme;
}

const getSystemTheme = (): string => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? "dark" : "light";
}

const App = () => {
    const [theme, setTheme] = React.useState<string>(parseTheme());
    
    const updateTheme = () => {
        setTheme(parseTheme());
    }

    React.useEffect(() => {
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
            updateTheme();
        });
    }, []);

    return (
        <MemoryRouter>
            <link rel="stylesheet" href={`${cfg.getStaticDir()}/themes/${theme}.css`} />
            <Layout sidebarWidth={48}>
                <Nav />
                <Content>
                    <Routes onThemeChange={updateTheme} />
                </Content>
            </Layout>
        </MemoryRouter>
    );
}

export default App;