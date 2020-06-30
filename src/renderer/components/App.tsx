import * as React from 'react';
import { MemoryRouter } from 'react-router-dom';

import cfg from '../model/Config';

import Layout, { Content } from './Layout';
import Nav from './Nav';
import Routes from './Routes';

const App = () => {
    const [theme, setTheme] = React.useState<string>(cfg.getTheme());
    
    const updateTheme = () => {
        setTheme(cfg.getTheme());
    }

    return (
        <MemoryRouter>
            <div className={`${theme}-theme`}>
                <Layout sidebarWidth={48}>
                    <Nav />
                    <Content>
                        <Routes onThemeChange={updateTheme} />
                    </Content>
                </Layout>
            </div>
        </MemoryRouter>
    );
}

export default App;