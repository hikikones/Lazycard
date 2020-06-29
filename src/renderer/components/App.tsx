import * as React from 'react';
import { MemoryRouter } from 'react-router-dom';

import Layout, { Content } from './Layout';
import Nav from './Nav';
import Main from './Main';

const App = () => {
    return (
        <MemoryRouter>
            <Layout sidebarWidth={48}>
                <Nav />
                <Content>
                    <Main />
                </Content>
            </Layout>
        </MemoryRouter>
    );
}

export default App;