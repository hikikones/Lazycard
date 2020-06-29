import * as React from 'react';
import { MemoryRouter } from 'react-router-dom';

import Layout, { Content } from './Layout';
import Nav from './Nav';
import Routes from './Routes';

const App = () => {
    return (
        <MemoryRouter>
            <Layout sidebarWidth={48}>
                <Nav />
                <Content>
                    <Routes />
                </Content>
            </Layout>
        </MemoryRouter>
    );
}

export default App;