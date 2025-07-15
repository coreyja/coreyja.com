import { jsxs, jsx, Fragment } from 'react/jsx-runtime';
import { memo, useState, useCallback, forwardRef, useRef, useContext, createContext, useMemo, useEffect } from 'react';
import cc from 'classcat';
import { drag } from 'd3-drag';
import { select, pointer } from 'd3-selection';
import { zoom, zoomIdentity, zoomTransform } from 'd3-zoom';
import { interpolateZoom, interpolate } from 'd3-interpolate';
import { useStoreWithEqualityFn, createWithEqualityFn } from 'zustand/traditional';
import { shallow } from 'zustand/shallow';
import axios from 'axios';

const errorMessages = {
    error001: () => '[React Flow]: Seems like you have not used zustand provider as an ancestor. Help: https://reactflow.dev/error#001',
    error002: () => "It looks like you've created a new nodeTypes or edgeTypes object. If this wasn't on purpose please define the nodeTypes/edgeTypes outside of the component or memoize them.",
    error003: (nodeType) => `Node type "${nodeType}" not found. Using fallback type "default".`,
    error004: () => 'The React Flow parent container needs a width and a height to render the graph.',
    error005: () => 'Only child nodes can use a parent extent.',
    error006: () => "Can't create edge. An edge needs a source and a target.",
    error007: (id) => `The old edge with id=${id} does not exist.`,
    error009: (type) => `Marker type "${type}" doesn't exist.`,
    error008: (handleType, { id, sourceHandle, targetHandle }) => `Couldn't create edge for ${handleType} handle id: "${handleType === 'source' ? sourceHandle : targetHandle}", edge id: ${id}.`,
    error010: () => 'Handle: No node id found. Make sure to only use a Handle inside a custom Node.',
    error011: (edgeType) => `Edge type "${edgeType}" not found. Using fallback type "default".`,
    error012: (id) => `Node with id "${id}" does not exist, it may have been removed. This can happen when a node is deleted before the "onNodeClick" handler is called.`,
    error013: (lib = 'react') => `It seems that you haven't loaded the styles. Please import '@xyflow/${lib}/dist/style.css' or base.css to make sure everything is working properly.`,
    error014: () => 'useNodeConnections: No node ID found. Call useNodeConnections inside a custom Node or provide a node ID.',
    error015: () => 'It seems that you are trying to drag a node that is not initialized. Please use onNodesChange as explained in the docs.',
};
const infiniteExtent = [
    [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
    [Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY],
];
const elementSelectionKeys = ['Enter', ' ', 'Escape'];
const defaultAriaLabelConfig = {
    'node.a11yDescription.default': 'Press enter or space to select a node. Press delete to remove it and escape to cancel.',
    'node.a11yDescription.keyboardDisabled': 'Press enter or space to select a node. You can then use the arrow keys to move the node around. Press delete to remove it and escape to cancel.',
    'node.a11yDescription.ariaLiveMessage': ({ direction, x, y }) => `Moved selected node ${direction}. New position, x: ${x}, y: ${y}`,
    'edge.a11yDescription.default': 'Press enter or space to select an edge. You can then press delete to remove it or escape to cancel.',
    // Control elements
    'controls.ariaLabel': 'Control Panel',
    'controls.zoomIn.ariaLabel': 'Zoom In',
    'controls.zoomOut.ariaLabel': 'Zoom Out',
    'controls.fitView.ariaLabel': 'Fit View',
    'controls.interactive.ariaLabel': 'Toggle Interactivity',
    // Mini map
    'minimap.ariaLabel': 'Mini Map',
    // Handle
    'handle.ariaLabel': 'Handle',
};

/**
 * The `ConnectionMode` is used to set the mode of connection between nodes.
 * The `Strict` mode is the default one and only allows source to target edges.
 * `Loose` mode allows source to source and target to target edges as well.
 *
 * @public
 */
var ConnectionMode;
(function (ConnectionMode) {
    ConnectionMode["Strict"] = "strict";
    ConnectionMode["Loose"] = "loose";
})(ConnectionMode || (ConnectionMode = {}));
/**
 * This enum is used to set the different modes of panning the viewport when the
 * user scrolls. The `Free` mode allows the user to pan in any direction by scrolling
 * with a device like a trackpad. The `Vertical` and `Horizontal` modes restrict
 * scroll panning to only the vertical or horizontal axis, respectively.
 *
 * @public
 */
var PanOnScrollMode;
(function (PanOnScrollMode) {
    PanOnScrollMode["Free"] = "free";
    PanOnScrollMode["Vertical"] = "vertical";
    PanOnScrollMode["Horizontal"] = "horizontal";
})(PanOnScrollMode || (PanOnScrollMode = {}));
var SelectionMode;
(function (SelectionMode) {
    SelectionMode["Partial"] = "partial";
    SelectionMode["Full"] = "full";
})(SelectionMode || (SelectionMode = {}));
const initialConnection = {
    inProgress: false,
    isValid: null,
    from: null,
    fromHandle: null,
    fromPosition: null,
    fromNode: null,
    to: null,
    toHandle: null,
    toPosition: null,
    toNode: null,
};

/**
 * If you set the `connectionLineType` prop on your [`<ReactFlow />`](/api-reference/react-flow#connection-connectionLineType)
 *component, it will dictate the style of connection line rendered when creating
 *new edges.
 *
 * @public
 *
 * @remarks If you choose to render a custom connection line component, this value will be
 *passed to your component as part of its [`ConnectionLineComponentProps`](/api-reference/types/connection-line-component-props).
 */
var ConnectionLineType;
(function (ConnectionLineType) {
    ConnectionLineType["Bezier"] = "default";
    ConnectionLineType["Straight"] = "straight";
    ConnectionLineType["Step"] = "step";
    ConnectionLineType["SmoothStep"] = "smoothstep";
    ConnectionLineType["SimpleBezier"] = "simplebezier";
})(ConnectionLineType || (ConnectionLineType = {}));
/**
 * Edges may optionally have a marker on either end. The MarkerType type enumerates
 * the options available to you when configuring a given marker.
 *
 * @public
 */
var MarkerType;
(function (MarkerType) {
    MarkerType["Arrow"] = "arrow";
    MarkerType["ArrowClosed"] = "arrowclosed";
})(MarkerType || (MarkerType = {}));

/**
 * While [`PanelPosition`](/api-reference/types/panel-position) can be used to place a
 * component in the corners of a container, the `Position` enum is less precise and used
 * primarily in relation to edges and handles.
 *
 * @public
 */
var Position;
(function (Position) {
    Position["Left"] = "left";
    Position["Top"] = "top";
    Position["Right"] = "right";
    Position["Bottom"] = "bottom";
})(Position || (Position = {}));
const oppositePosition = {
    [Position.Left]: Position.Right,
    [Position.Right]: Position.Left,
    [Position.Top]: Position.Bottom,
    [Position.Bottom]: Position.Top,
};
function getConnectionStatus(isValid) {
    return isValid === null ? null : isValid ? 'valid' : 'invalid';
}

/* eslint-disable @typescript-eslint/no-explicit-any */
/**
 * Test whether an object is usable as an Edge
 * @public
 * @remarks In TypeScript this is a type guard that will narrow the type of whatever you pass in to Edge if it returns true
 * @param element - The element to test
 * @returns A boolean indicating whether the element is an Edge
 */
const isEdgeBase = (element) => 'id' in element && 'source' in element && 'target' in element;
/**
 * Test whether an object is usable as a Node
 * @public
 * @remarks In TypeScript this is a type guard that will narrow the type of whatever you pass in to Node if it returns true
 * @param element - The element to test
 * @returns A boolean indicating whether the element is an Node
 */
const isNodeBase = (element) => 'id' in element && 'position' in element && !('source' in element) && !('target' in element);
const isInternalNodeBase = (element) => 'id' in element && 'internals' in element && !('source' in element) && !('target' in element);
const getNodePositionWithOrigin = (node, nodeOrigin = [0, 0]) => {
    const { width, height } = getNodeDimensions(node);
    const origin = node.origin ?? nodeOrigin;
    const offsetX = width * origin[0];
    const offsetY = height * origin[1];
    return {
        x: node.position.x - offsetX,
        y: node.position.y - offsetY,
    };
};
/**
 * Returns the bounding box that contains all the given nodes in an array. This can
 * be useful when combined with [`getViewportForBounds`](/api-reference/utils/get-viewport-for-bounds)
 * to calculate the correct transform to fit the given nodes in a viewport.
 * @public
 * @remarks Useful when combined with {@link getViewportForBounds} to calculate the correct transform to fit the given nodes in a viewport.
 * @param nodes - Nodes to calculate the bounds for.
 * @returns Bounding box enclosing all nodes.
 *
 * @remarks This function was previously called `getRectOfNodes`
 *
 * @example
 * ```js
 *import { getNodesBounds } from '@xyflow/react';
 *
 *const nodes = [
 *  {
 *    id: 'a',
 *    position: { x: 0, y: 0 },
 *    data: { label: 'a' },
 *    width: 50,
 *    height: 25,
 *  },
 *  {
 *    id: 'b',
 *    position: { x: 100, y: 100 },
 *    data: { label: 'b' },
 *    width: 50,
 *    height: 25,
 *  },
 *];
 *
 *const bounds = getNodesBounds(nodes);
 *```
 */
const getNodesBounds = (nodes, params = { nodeOrigin: [0, 0] }) => {
    if (nodes.length === 0) {
        return { x: 0, y: 0, width: 0, height: 0 };
    }
    const box = nodes.reduce((currBox, nodeOrId) => {
        const isId = typeof nodeOrId === 'string';
        let currentNode = !params.nodeLookup && !isId ? nodeOrId : undefined;
        if (params.nodeLookup) {
            currentNode = isId
                ? params.nodeLookup.get(nodeOrId)
                : !isInternalNodeBase(nodeOrId)
                    ? params.nodeLookup.get(nodeOrId.id)
                    : nodeOrId;
        }
        const nodeBox = currentNode ? nodeToBox(currentNode, params.nodeOrigin) : { x: 0, y: 0, x2: 0, y2: 0 };
        return getBoundsOfBoxes(currBox, nodeBox);
    }, { x: Infinity, y: Infinity, x2: -Infinity, y2: -Infinity });
    return boxToRect(box);
};
/**
 * Determines a bounding box that contains all given nodes in an array
 * @internal
 */
const getInternalNodesBounds = (nodeLookup, params = {}) => {
    if (nodeLookup.size === 0) {
        return { x: 0, y: 0, width: 0, height: 0 };
    }
    let box = { x: Infinity, y: Infinity, x2: -Infinity, y2: -Infinity };
    nodeLookup.forEach((node) => {
        if (params.filter === undefined || params.filter(node)) {
            const nodeBox = nodeToBox(node);
            box = getBoundsOfBoxes(box, nodeBox);
        }
    });
    return boxToRect(box);
};
const getNodesInside = (nodes, rect, [tx, ty, tScale] = [0, 0, 1], partially = false, 
// set excludeNonSelectableNodes if you want to pay attention to the nodes "selectable" attribute
excludeNonSelectableNodes = false) => {
    const paneRect = {
        ...pointToRendererPoint(rect, [tx, ty, tScale]),
        width: rect.width / tScale,
        height: rect.height / tScale,
    };
    const visibleNodes = [];
    for (const node of nodes.values()) {
        const { measured, selectable = true, hidden = false } = node;
        if ((excludeNonSelectableNodes && !selectable) || hidden) {
            continue;
        }
        const width = measured.width ?? node.width ?? node.initialWidth ?? null;
        const height = measured.height ?? node.height ?? node.initialHeight ?? null;
        const overlappingArea = getOverlappingArea(paneRect, nodeToRect(node));
        const area = (width ?? 0) * (height ?? 0);
        const partiallyVisible = partially && overlappingArea > 0;
        const forceInitialRender = !node.internals.handleBounds;
        const isVisible = forceInitialRender || partiallyVisible || overlappingArea >= area;
        if (isVisible || node.dragging) {
            visibleNodes.push(node);
        }
    }
    return visibleNodes;
};
/**
 * This utility filters an array of edges, keeping only those where either the source or target
 * node is present in the given array of nodes.
 * @public
 * @param nodes - Nodes you want to get the connected edges for.
 * @param edges - All edges.
 * @returns Array of edges that connect any of the given nodes with each other.
 *
 * @example
 * ```js
 *import { getConnectedEdges } from '@xyflow/react';
 *
 *const nodes = [
 *  { id: 'a', position: { x: 0, y: 0 } },
 *  { id: 'b', position: { x: 100, y: 0 } },
 *];
 *
 *const edges = [
 *  { id: 'a->c', source: 'a', target: 'c' },
 *  { id: 'c->d', source: 'c', target: 'd' },
 *];
 *
 *const connectedEdges = getConnectedEdges(nodes, edges);
 * // => [{ id: 'a->c', source: 'a', target: 'c' }]
 *```
 */
const getConnectedEdges = (nodes, edges) => {
    const nodeIds = new Set();
    nodes.forEach((node) => {
        nodeIds.add(node.id);
    });
    return edges.filter((edge) => nodeIds.has(edge.source) || nodeIds.has(edge.target));
};
function getFitViewNodes(nodeLookup, options) {
    const fitViewNodes = new Map();
    const optionNodeIds = options?.nodes ? new Set(options.nodes.map((node) => node.id)) : null;
    nodeLookup.forEach((n) => {
        const isVisible = n.measured.width && n.measured.height && (options?.includeHiddenNodes || !n.hidden);
        if (isVisible && (!optionNodeIds || optionNodeIds.has(n.id))) {
            fitViewNodes.set(n.id, n);
        }
    });
    return fitViewNodes;
}
async function fitViewport({ nodes, width, height, panZoom, minZoom, maxZoom }, options) {
    if (nodes.size === 0) {
        return Promise.resolve(true);
    }
    const nodesToFit = getFitViewNodes(nodes, options);
    const bounds = getInternalNodesBounds(nodesToFit);
    const viewport = getViewportForBounds(bounds, width, height, options?.minZoom ?? minZoom, options?.maxZoom ?? maxZoom, options?.padding ?? 0.1);
    await panZoom.setViewport(viewport, {
        duration: options?.duration,
        ease: options?.ease,
        interpolate: options?.interpolate,
    });
    return Promise.resolve(true);
}
/**
 * This function calculates the next position of a node, taking into account the node's extent, parent node, and origin.
 *
 * @internal
 * @returns position, positionAbsolute
 */
function calculateNodePosition({ nodeId, nextPosition, nodeLookup, nodeOrigin = [0, 0], nodeExtent, onError, }) {
    const node = nodeLookup.get(nodeId);
    const parentNode = node.parentId ? nodeLookup.get(node.parentId) : undefined;
    const { x: parentX, y: parentY } = parentNode ? parentNode.internals.positionAbsolute : { x: 0, y: 0 };
    const origin = node.origin ?? nodeOrigin;
    let extent = node.extent || nodeExtent;
    if (node.extent === 'parent' && !node.expandParent) {
        if (!parentNode) {
            onError?.('005', errorMessages['error005']());
        }
        else {
            const parentWidth = parentNode.measured.width;
            const parentHeight = parentNode.measured.height;
            if (parentWidth && parentHeight) {
                extent = [
                    [parentX, parentY],
                    [parentX + parentWidth, parentY + parentHeight],
                ];
            }
        }
    }
    else if (parentNode && isCoordinateExtent(node.extent)) {
        extent = [
            [node.extent[0][0] + parentX, node.extent[0][1] + parentY],
            [node.extent[1][0] + parentX, node.extent[1][1] + parentY],
        ];
    }
    const positionAbsolute = isCoordinateExtent(extent)
        ? clampPosition(nextPosition, extent, node.measured)
        : nextPosition;
    if (node.measured.width === undefined || node.measured.height === undefined) {
        onError?.('015', errorMessages['error015']());
    }
    return {
        position: {
            x: positionAbsolute.x - parentX + (node.measured.width ?? 0) * origin[0],
            y: positionAbsolute.y - parentY + (node.measured.height ?? 0) * origin[1],
        },
        positionAbsolute,
    };
}
/**
 * Pass in nodes & edges to delete, get arrays of nodes and edges that actually can be deleted
 * @internal
 * @param param.nodesToRemove - The nodes to remove
 * @param param.edgesToRemove - The edges to remove
 * @param param.nodes - All nodes
 * @param param.edges - All edges
 * @param param.onBeforeDelete - Callback to check which nodes and edges can be deleted
 * @returns nodes: nodes that can be deleted, edges: edges that can be deleted
 */
async function getElementsToRemove({ nodesToRemove = [], edgesToRemove = [], nodes, edges, onBeforeDelete, }) {
    const nodeIds = new Set(nodesToRemove.map((node) => node.id));
    const matchingNodes = [];
    for (const node of nodes) {
        if (node.deletable === false) {
            continue;
        }
        const isIncluded = nodeIds.has(node.id);
        const parentHit = !isIncluded && node.parentId && matchingNodes.find((n) => n.id === node.parentId);
        if (isIncluded || parentHit) {
            matchingNodes.push(node);
        }
    }
    const edgeIds = new Set(edgesToRemove.map((edge) => edge.id));
    const deletableEdges = edges.filter((edge) => edge.deletable !== false);
    const connectedEdges = getConnectedEdges(matchingNodes, deletableEdges);
    const matchingEdges = connectedEdges;
    for (const edge of deletableEdges) {
        const isIncluded = edgeIds.has(edge.id);
        if (isIncluded && !matchingEdges.find((e) => e.id === edge.id)) {
            matchingEdges.push(edge);
        }
    }
    if (!onBeforeDelete) {
        return {
            edges: matchingEdges,
            nodes: matchingNodes,
        };
    }
    const onBeforeDeleteResult = await onBeforeDelete({
        nodes: matchingNodes,
        edges: matchingEdges,
    });
    if (typeof onBeforeDeleteResult === 'boolean') {
        return onBeforeDeleteResult ? { edges: matchingEdges, nodes: matchingNodes } : { edges: [], nodes: [] };
    }
    return onBeforeDeleteResult;
}

const clamp = (val, min = 0, max = 1) => Math.min(Math.max(val, min), max);
const clampPosition = (position = { x: 0, y: 0 }, extent, dimensions) => ({
    x: clamp(position.x, extent[0][0], extent[1][0] - (dimensions?.width ?? 0)),
    y: clamp(position.y, extent[0][1], extent[1][1] - (dimensions?.height ?? 0)),
});
function clampPositionToParent(childPosition, childDimensions, parent) {
    const { width: parentWidth, height: parentHeight } = getNodeDimensions(parent);
    const { x: parentX, y: parentY } = parent.internals.positionAbsolute;
    return clampPosition(childPosition, [
        [parentX, parentY],
        [parentX + parentWidth, parentY + parentHeight],
    ], childDimensions);
}
/**
 * Calculates the velocity of panning when the mouse is close to the edge of the canvas
 * @internal
 * @param value - One dimensional poition of the mouse (x or y)
 * @param min - Minimal position on canvas before panning starts
 * @param max - Maximal position on canvas before panning starts
 * @returns - A number between 0 and 1 that represents the velocity of panning
 */
const calcAutoPanVelocity = (value, min, max) => {
    if (value < min) {
        return clamp(Math.abs(value - min), 1, min) / min;
    }
    else if (value > max) {
        return -clamp(Math.abs(value - max), 1, min) / min;
    }
    return 0;
};
const calcAutoPan = (pos, bounds, speed = 15, distance = 40) => {
    const xMovement = calcAutoPanVelocity(pos.x, distance, bounds.width - distance) * speed;
    const yMovement = calcAutoPanVelocity(pos.y, distance, bounds.height - distance) * speed;
    return [xMovement, yMovement];
};
const getBoundsOfBoxes = (box1, box2) => ({
    x: Math.min(box1.x, box2.x),
    y: Math.min(box1.y, box2.y),
    x2: Math.max(box1.x2, box2.x2),
    y2: Math.max(box1.y2, box2.y2),
});
const rectToBox = ({ x, y, width, height }) => ({
    x,
    y,
    x2: x + width,
    y2: y + height,
});
const boxToRect = ({ x, y, x2, y2 }) => ({
    x,
    y,
    width: x2 - x,
    height: y2 - y,
});
const nodeToRect = (node, nodeOrigin = [0, 0]) => {
    const { x, y } = isInternalNodeBase(node)
        ? node.internals.positionAbsolute
        : getNodePositionWithOrigin(node, nodeOrigin);
    return {
        x,
        y,
        width: node.measured?.width ?? node.width ?? node.initialWidth ?? 0,
        height: node.measured?.height ?? node.height ?? node.initialHeight ?? 0,
    };
};
const nodeToBox = (node, nodeOrigin = [0, 0]) => {
    const { x, y } = isInternalNodeBase(node)
        ? node.internals.positionAbsolute
        : getNodePositionWithOrigin(node, nodeOrigin);
    return {
        x,
        y,
        x2: x + (node.measured?.width ?? node.width ?? node.initialWidth ?? 0),
        y2: y + (node.measured?.height ?? node.height ?? node.initialHeight ?? 0),
    };
};
const getBoundsOfRects = (rect1, rect2) => boxToRect(getBoundsOfBoxes(rectToBox(rect1), rectToBox(rect2)));
const getOverlappingArea = (rectA, rectB) => {
    const xOverlap = Math.max(0, Math.min(rectA.x + rectA.width, rectB.x + rectB.width) - Math.max(rectA.x, rectB.x));
    const yOverlap = Math.max(0, Math.min(rectA.y + rectA.height, rectB.y + rectB.height) - Math.max(rectA.y, rectB.y));
    return Math.ceil(xOverlap * yOverlap);
};
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const isRectObject = (obj) => isNumeric(obj.width) && isNumeric(obj.height) && isNumeric(obj.x) && isNumeric(obj.y);
/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isNumeric = (n) => !isNaN(n) && isFinite(n);
// used for a11y key board controls for nodes and edges
const devWarn = (id, message) => {
};
const snapPosition = (position, snapGrid = [1, 1]) => {
    return {
        x: snapGrid[0] * Math.round(position.x / snapGrid[0]),
        y: snapGrid[1] * Math.round(position.y / snapGrid[1]),
    };
};
const pointToRendererPoint = ({ x, y }, [tx, ty, tScale], snapToGrid = false, snapGrid = [1, 1]) => {
    const position = {
        x: (x - tx) / tScale,
        y: (y - ty) / tScale,
    };
    return snapToGrid ? snapPosition(position, snapGrid) : position;
};
const rendererPointToPoint = ({ x, y }, [tx, ty, tScale]) => {
    return {
        x: x * tScale + tx,
        y: y * tScale + ty,
    };
};
/**
 * Parses a single padding value to a number
 * @internal
 * @param padding - Padding to parse
 * @param viewport - Width or height of the viewport
 * @returns The padding in pixels
 */
function parsePadding(padding, viewport) {
    if (typeof padding === 'number') {
        return Math.floor((viewport - viewport / (1 + padding)) * 0.5);
    }
    if (typeof padding === 'string' && padding.endsWith('px')) {
        const paddingValue = parseFloat(padding);
        if (!Number.isNaN(paddingValue)) {
            return Math.floor(paddingValue);
        }
    }
    if (typeof padding === 'string' && padding.endsWith('%')) {
        const paddingValue = parseFloat(padding);
        if (!Number.isNaN(paddingValue)) {
            return Math.floor(viewport * paddingValue * 0.01);
        }
    }
    console.error(`[React Flow] The padding value "${padding}" is invalid. Please provide a number or a string with a valid unit (px or %).`);
    return 0;
}
/**
 * Parses the paddings to an object with top, right, bottom, left, x and y paddings
 * @internal
 * @param padding - Padding to parse
 * @param width - Width of the viewport
 * @param height - Height of the viewport
 * @returns An object with the paddings in pixels
 */
function parsePaddings(padding, width, height) {
    if (typeof padding === 'string' || typeof padding === 'number') {
        const paddingY = parsePadding(padding, height);
        const paddingX = parsePadding(padding, width);
        return {
            top: paddingY,
            right: paddingX,
            bottom: paddingY,
            left: paddingX,
            x: paddingX * 2,
            y: paddingY * 2,
        };
    }
    if (typeof padding === 'object') {
        const top = parsePadding(padding.top ?? padding.y ?? 0, height);
        const bottom = parsePadding(padding.bottom ?? padding.y ?? 0, height);
        const left = parsePadding(padding.left ?? padding.x ?? 0, width);
        const right = parsePadding(padding.right ?? padding.x ?? 0, width);
        return { top, right, bottom, left, x: left + right, y: top + bottom };
    }
    return { top: 0, right: 0, bottom: 0, left: 0, x: 0, y: 0 };
}
/**
 * Calculates the resulting paddings if the new viewport is applied
 * @internal
 * @param bounds - Bounds to fit inside viewport
 * @param x - X position of the viewport
 * @param y - Y position of the viewport
 * @param zoom - Zoom level of the viewport
 * @param width - Width of the viewport
 * @param height - Height of the viewport
 * @returns An object with the minimum padding required to fit the bounds inside the viewport
 */
function calculateAppliedPaddings(bounds, x, y, zoom, width, height) {
    const { x: left, y: top } = rendererPointToPoint(bounds, [x, y, zoom]);
    const { x: boundRight, y: boundBottom } = rendererPointToPoint({ x: bounds.x + bounds.width, y: bounds.y + bounds.height }, [x, y, zoom]);
    const right = width - boundRight;
    const bottom = height - boundBottom;
    return {
        left: Math.floor(left),
        top: Math.floor(top),
        right: Math.floor(right),
        bottom: Math.floor(bottom),
    };
}
/**
 * Returns a viewport that encloses the given bounds with padding.
 * @public
 * @remarks You can determine bounds of nodes with {@link getNodesBounds} and {@link getBoundsOfRects}
 * @param bounds - Bounds to fit inside viewport.
 * @param width - Width of the viewport.
 * @param height  - Height of the viewport.
 * @param minZoom - Minimum zoom level of the resulting viewport.
 * @param maxZoom - Maximum zoom level of the resulting viewport.
 * @param padding - Padding around the bounds.
 * @returns A transformed {@link Viewport} that encloses the given bounds which you can pass to e.g. {@link setViewport}.
 * @example
 * const { x, y, zoom } = getViewportForBounds(
 * { x: 0, y: 0, width: 100, height: 100},
 * 1200, 800, 0.5, 2);
 */
const getViewportForBounds = (bounds, width, height, minZoom, maxZoom, padding) => {
    // First we resolve all the paddings to actual pixel values
    const p = parsePaddings(padding, width, height);
    const xZoom = (width - p.x) / bounds.width;
    const yZoom = (height - p.y) / bounds.height;
    // We calculate the new x, y, zoom for a centered view
    const zoom = Math.min(xZoom, yZoom);
    const clampedZoom = clamp(zoom, minZoom, maxZoom);
    const boundsCenterX = bounds.x + bounds.width / 2;
    const boundsCenterY = bounds.y + bounds.height / 2;
    const x = width / 2 - boundsCenterX * clampedZoom;
    const y = height / 2 - boundsCenterY * clampedZoom;
    // Then we calculate the minimum padding, to respect asymmetric paddings
    const newPadding = calculateAppliedPaddings(bounds, x, y, clampedZoom, width, height);
    // We only want to have an offset if the newPadding is smaller than the required padding
    const offset = {
        left: Math.min(newPadding.left - p.left, 0),
        top: Math.min(newPadding.top - p.top, 0),
        right: Math.min(newPadding.right - p.right, 0),
        bottom: Math.min(newPadding.bottom - p.bottom, 0),
    };
    return {
        x: x - offset.left + offset.right,
        y: y - offset.top + offset.bottom,
        zoom: clampedZoom,
    };
};
const isMacOs = () => typeof navigator !== 'undefined' && navigator?.userAgent?.indexOf('Mac') >= 0;
function isCoordinateExtent(extent) {
    return extent !== undefined && extent !== 'parent';
}
function getNodeDimensions(node) {
    return {
        width: node.measured?.width ?? node.width ?? node.initialWidth ?? 0,
        height: node.measured?.height ?? node.height ?? node.initialHeight ?? 0,
    };
}
function nodeHasDimensions(node) {
    return ((node.measured?.width ?? node.width ?? node.initialWidth) !== undefined &&
        (node.measured?.height ?? node.height ?? node.initialHeight) !== undefined);
}
/**
 * Convert child position to aboslute position
 *
 * @internal
 * @param position
 * @param parentId
 * @param nodeLookup
 * @param nodeOrigin
 * @returns an internal node with an absolute position
 */
function evaluateAbsolutePosition(position, dimensions = { width: 0, height: 0 }, parentId, nodeLookup, nodeOrigin) {
    const positionAbsolute = { ...position };
    const parent = nodeLookup.get(parentId);
    if (parent) {
        const origin = parent.origin || nodeOrigin;
        positionAbsolute.x += parent.internals.positionAbsolute.x - (dimensions.width ?? 0) * origin[0];
        positionAbsolute.y += parent.internals.positionAbsolute.y - (dimensions.height ?? 0) * origin[1];
    }
    return positionAbsolute;
}
function areSetsEqual(a, b) {
    if (a.size !== b.size) {
        return false;
    }
    for (const item of a) {
        if (!b.has(item)) {
            return false;
        }
    }
    return true;
}
/**
 * Polyfill for Promise.withResolvers until we can use it in all browsers
 * @internal
 */
function withResolvers() {
    let resolve;
    let reject;
    const promise = new Promise((res, rej) => {
        resolve = res;
        reject = rej;
    });
    return { promise, resolve, reject };
}
function mergeAriaLabelConfig(partial) {
    return { ...defaultAriaLabelConfig, ...(partial || {}) };
}

function getPointerPosition(event, { snapGrid = [0, 0], snapToGrid = false, transform, containerBounds }) {
    const { x, y } = getEventPosition(event);
    const pointerPos = pointToRendererPoint({ x: x - (containerBounds?.left ?? 0), y: y - (containerBounds?.top ?? 0) }, transform);
    const { x: xSnapped, y: ySnapped } = snapToGrid ? snapPosition(pointerPos, snapGrid) : pointerPos;
    // we need the snapped position in order to be able to skip unnecessary drag events
    return {
        xSnapped,
        ySnapped,
        ...pointerPos,
    };
}
const getDimensions = (node) => ({
    width: node.offsetWidth,
    height: node.offsetHeight,
});
const getHostForElement = (element) => element?.getRootNode?.() || window?.document;
const inputTags = ['INPUT', 'SELECT', 'TEXTAREA'];
function isInputDOMNode(event) {
    // using composed path for handling shadow dom
    const target = (event.composedPath?.()?.[0] || event.target);
    if (target?.nodeType !== 1 /* Node.ELEMENT_NODE */)
        return false;
    const isInput = inputTags.includes(target.nodeName) || target.hasAttribute('contenteditable');
    // when an input field is focused we don't want to trigger deletion or movement of nodes
    return isInput || !!target.closest('.nokey');
}
const isMouseEvent = (event) => 'clientX' in event;
const getEventPosition = (event, bounds) => {
    const isMouse = isMouseEvent(event);
    const evtX = isMouse ? event.clientX : event.touches?.[0].clientX;
    const evtY = isMouse ? event.clientY : event.touches?.[0].clientY;
    return {
        x: evtX - (bounds?.left ?? 0),
        y: evtY - (bounds?.top ?? 0),
    };
};
/*
 * The handle bounds are calculated relative to the node element.
 * We store them in the internals object of the node in order to avoid
 * unnecessary recalculations.
 */
const getHandleBounds = (type, nodeElement, nodeBounds, zoom, nodeId) => {
    const handles = nodeElement.querySelectorAll(`.${type}`);
    if (!handles || !handles.length) {
        return null;
    }
    return Array.from(handles).map((handle) => {
        const handleBounds = handle.getBoundingClientRect();
        return {
            id: handle.getAttribute('data-handleid'),
            type,
            nodeId,
            position: handle.getAttribute('data-handlepos'),
            x: (handleBounds.left - nodeBounds.left) / zoom,
            y: (handleBounds.top - nodeBounds.top) / zoom,
            ...getDimensions(handle),
        };
    });
};

function getBezierEdgeCenter({ sourceX, sourceY, targetX, targetY, sourceControlX, sourceControlY, targetControlX, targetControlY, }) {
    /*
     * cubic bezier t=0.5 mid point, not the actual mid point, but easy to calculate
     * https://stackoverflow.com/questions/67516101/how-to-find-distance-mid-point-of-bezier-curve
     */
    const centerX = sourceX * 0.125 + sourceControlX * 0.375 + targetControlX * 0.375 + targetX * 0.125;
    const centerY = sourceY * 0.125 + sourceControlY * 0.375 + targetControlY * 0.375 + targetY * 0.125;
    const offsetX = Math.abs(centerX - sourceX);
    const offsetY = Math.abs(centerY - sourceY);
    return [centerX, centerY, offsetX, offsetY];
}
function calculateControlOffset(distance, curvature) {
    if (distance >= 0) {
        return 0.5 * distance;
    }
    return curvature * 25 * Math.sqrt(-distance);
}
function getControlWithCurvature({ pos, x1, y1, x2, y2, c }) {
    switch (pos) {
        case Position.Left:
            return [x1 - calculateControlOffset(x1 - x2, c), y1];
        case Position.Right:
            return [x1 + calculateControlOffset(x2 - x1, c), y1];
        case Position.Top:
            return [x1, y1 - calculateControlOffset(y1 - y2, c)];
        case Position.Bottom:
            return [x1, y1 + calculateControlOffset(y2 - y1, c)];
    }
}
/**
 * The `getBezierPath` util returns everything you need to render a bezier edge
 *between two nodes.
 * @public
 * @returns A path string you can use in an SVG, the `labelX` and `labelY` position (center of path)
 * and `offsetX`, `offsetY` between source handle and label.
 * - `path`: the path to use in an SVG `<path>` element.
 * - `labelX`: the `x` position you can use to render a label for this edge.
 * - `labelY`: the `y` position you can use to render a label for this edge.
 * - `offsetX`: the absolute difference between the source `x` position and the `x` position of the
 * middle of this path.
 * - `offsetY`: the absolute difference between the source `y` position and the `y` position of the
 * middle of this path.
 * @example
 * ```js
 *  const source = { x: 0, y: 20 };
 *  const target = { x: 150, y: 100 };
 *
 *  const [path, labelX, labelY, offsetX, offsetY] = getBezierPath({
 *    sourceX: source.x,
 *    sourceY: source.y,
 *    sourcePosition: Position.Right,
 *    targetX: target.x,
 *    targetY: target.y,
 *    targetPosition: Position.Left,
 *});
 *```
 *
 * @remarks This function returns a tuple (aka a fixed-size array) to make it easier to
 *work with multiple edge paths at once.
 */
function getBezierPath({ sourceX, sourceY, sourcePosition = Position.Bottom, targetX, targetY, targetPosition = Position.Top, curvature = 0.25, }) {
    const [sourceControlX, sourceControlY] = getControlWithCurvature({
        pos: sourcePosition,
        x1: sourceX,
        y1: sourceY,
        x2: targetX,
        y2: targetY,
        c: curvature,
    });
    const [targetControlX, targetControlY] = getControlWithCurvature({
        pos: targetPosition,
        x1: targetX,
        y1: targetY,
        x2: sourceX,
        y2: sourceY,
        c: curvature,
    });
    const [labelX, labelY, offsetX, offsetY] = getBezierEdgeCenter({
        sourceX,
        sourceY,
        targetX,
        targetY,
        sourceControlX,
        sourceControlY,
        targetControlX,
        targetControlY,
    });
    return [
        `M${sourceX},${sourceY} C${sourceControlX},${sourceControlY} ${targetControlX},${targetControlY} ${targetX},${targetY}`,
        labelX,
        labelY,
        offsetX,
        offsetY,
    ];
}

// this is used for straight edges and simple smoothstep edges (LTR, RTL, BTT, TTB)
function getEdgeCenter({ sourceX, sourceY, targetX, targetY, }) {
    const xOffset = Math.abs(targetX - sourceX) / 2;
    const centerX = targetX < sourceX ? targetX + xOffset : targetX - xOffset;
    const yOffset = Math.abs(targetY - sourceY) / 2;
    const centerY = targetY < sourceY ? targetY + yOffset : targetY - yOffset;
    return [centerX, centerY, xOffset, yOffset];
}
/**
 * Returns the z-index for an edge based on the node it connects and whether it is selected.
 * By default, edges are rendered below nodes. This behaviour is different for edges that are
 * connected to nodes with a parent, as they are rendered above the parent node.
 */
function getElevatedEdgeZIndex({ sourceNode, targetNode, selected = false, zIndex, elevateOnSelect = false, }) {
    if (zIndex !== undefined) {
        return zIndex;
    }
    const edgeZ = elevateOnSelect && selected ? 1000 : 0;
    const nodeZ = Math.max(sourceNode.parentId ? sourceNode.internals.z : 0, targetNode.parentId ? targetNode.internals.z : 0);
    return edgeZ + nodeZ;
}
function isEdgeVisible({ sourceNode, targetNode, width, height, transform }) {
    const edgeBox = getBoundsOfBoxes(nodeToBox(sourceNode), nodeToBox(targetNode));
    if (edgeBox.x === edgeBox.x2) {
        edgeBox.x2 += 1;
    }
    if (edgeBox.y === edgeBox.y2) {
        edgeBox.y2 += 1;
    }
    const viewRect = {
        x: -transform[0] / transform[2],
        y: -transform[1] / transform[2],
        width: width / transform[2],
        height: height / transform[2],
    };
    return getOverlappingArea(viewRect, boxToRect(edgeBox)) > 0;
}
const getEdgeId = ({ source, sourceHandle, target, targetHandle }) => `xy-edge__${source}${sourceHandle || ''}-${target}${targetHandle || ''}`;
const connectionExists = (edge, edges) => {
    return edges.some((el) => el.source === edge.source &&
        el.target === edge.target &&
        (el.sourceHandle === edge.sourceHandle || (!el.sourceHandle && !edge.sourceHandle)) &&
        (el.targetHandle === edge.targetHandle || (!el.targetHandle && !edge.targetHandle)));
};
/**
 * This util is a convenience function to add a new Edge to an array of edges. It also performs some validation to make sure you don't add an invalid edge or duplicate an existing one.
 * @public
 * @param edgeParams - Either an `Edge` or a `Connection` you want to add.
 * @param edges - The array of all current edges.
 * @returns A new array of edges with the new edge added.
 *
 * @remarks If an edge with the same `target` and `source` already exists (and the same
 *`targetHandle` and `sourceHandle` if those are set), then this util won't add
 *a new edge even if the `id` property is different.
 *
 */
const addEdge = (edgeParams, edges) => {
    if (!edgeParams.source || !edgeParams.target) {
        return edges;
    }
    let edge;
    if (isEdgeBase(edgeParams)) {
        edge = { ...edgeParams };
    }
    else {
        edge = {
            ...edgeParams,
            id: getEdgeId(edgeParams),
        };
    }
    if (connectionExists(edge, edges)) {
        return edges;
    }
    if (edge.sourceHandle === null) {
        delete edge.sourceHandle;
    }
    if (edge.targetHandle === null) {
        delete edge.targetHandle;
    }
    return edges.concat(edge);
};

/**
 * Calculates the straight line path between two points.
 * @public
 * @returns A path string you can use in an SVG, the `labelX` and `labelY` position (center of path)
 * and `offsetX`, `offsetY` between source handle and label.
 *
 * - `path`: the path to use in an SVG `<path>` element.
 * - `labelX`: the `x` position you can use to render a label for this edge.
 * - `labelY`: the `y` position you can use to render a label for this edge.
 * - `offsetX`: the absolute difference between the source `x` position and the `x` position of the
 * middle of this path.
 * - `offsetY`: the absolute difference between the source `y` position and the `y` position of the
 * middle of this path.
 * @example
 * ```js
 *  const source = { x: 0, y: 20 };
 *  const target = { x: 150, y: 100 };
 *
 *  const [path, labelX, labelY, offsetX, offsetY] = getStraightPath({
 *    sourceX: source.x,
 *    sourceY: source.y,
 *    sourcePosition: Position.Right,
 *    targetX: target.x,
 *    targetY: target.y,
 *    targetPosition: Position.Left,
 *  });
 * ```
 * @remarks This function returns a tuple (aka a fixed-size array) to make it easier to work with multiple edge paths at once.
 */
function getStraightPath({ sourceX, sourceY, targetX, targetY, }) {
    const [labelX, labelY, offsetX, offsetY] = getEdgeCenter({
        sourceX,
        sourceY,
        targetX,
        targetY,
    });
    return [`M ${sourceX},${sourceY}L ${targetX},${targetY}`, labelX, labelY, offsetX, offsetY];
}

const handleDirections = {
    [Position.Left]: { x: -1, y: 0 },
    [Position.Right]: { x: 1, y: 0 },
    [Position.Top]: { x: 0, y: -1 },
    [Position.Bottom]: { x: 0, y: 1 },
};
const getDirection = ({ source, sourcePosition = Position.Bottom, target, }) => {
    if (sourcePosition === Position.Left || sourcePosition === Position.Right) {
        return source.x < target.x ? { x: 1, y: 0 } : { x: -1, y: 0 };
    }
    return source.y < target.y ? { x: 0, y: 1 } : { x: 0, y: -1 };
};
const distance = (a, b) => Math.sqrt(Math.pow(b.x - a.x, 2) + Math.pow(b.y - a.y, 2));
/*
 * With this function we try to mimic an orthogonal edge routing behaviour
 * It's not as good as a real orthogonal edge routing, but it's faster and good enough as a default for step and smooth step edges
 */
function getPoints({ source, sourcePosition = Position.Bottom, target, targetPosition = Position.Top, center, offset, stepPosition, }) {
    const sourceDir = handleDirections[sourcePosition];
    const targetDir = handleDirections[targetPosition];
    const sourceGapped = { x: source.x + sourceDir.x * offset, y: source.y + sourceDir.y * offset };
    const targetGapped = { x: target.x + targetDir.x * offset, y: target.y + targetDir.y * offset };
    const dir = getDirection({
        source: sourceGapped,
        sourcePosition,
        target: targetGapped,
    });
    const dirAccessor = dir.x !== 0 ? 'x' : 'y';
    const currDir = dir[dirAccessor];
    let points = [];
    let centerX, centerY;
    const sourceGapOffset = { x: 0, y: 0 };
    const targetGapOffset = { x: 0, y: 0 };
    const [, , defaultOffsetX, defaultOffsetY] = getEdgeCenter({
        sourceX: source.x,
        sourceY: source.y,
        targetX: target.x,
        targetY: target.y,
    });
    // opposite handle positions, default case
    if (sourceDir[dirAccessor] * targetDir[dirAccessor] === -1) {
        if (dirAccessor === 'x') {
            // Primary direction is horizontal, so stepPosition affects X coordinate
            centerX = center.x ?? (sourceGapped.x + (targetGapped.x - sourceGapped.x) * stepPosition);
            centerY = center.y ?? (sourceGapped.y + targetGapped.y) / 2;
        }
        else {
            // Primary direction is vertical, so stepPosition affects Y coordinate  
            centerX = center.x ?? (sourceGapped.x + targetGapped.x) / 2;
            centerY = center.y ?? (sourceGapped.y + (targetGapped.y - sourceGapped.y) * stepPosition);
        }
        /*
         *    --->
         *    |
         * >---
         */
        const verticalSplit = [
            { x: centerX, y: sourceGapped.y },
            { x: centerX, y: targetGapped.y },
        ];
        /*
         *    |
         *  ---
         *  |
         */
        const horizontalSplit = [
            { x: sourceGapped.x, y: centerY },
            { x: targetGapped.x, y: centerY },
        ];
        if (sourceDir[dirAccessor] === currDir) {
            points = dirAccessor === 'x' ? verticalSplit : horizontalSplit;
        }
        else {
            points = dirAccessor === 'x' ? horizontalSplit : verticalSplit;
        }
    }
    else {
        // sourceTarget means we take x from source and y from target, targetSource is the opposite
        const sourceTarget = [{ x: sourceGapped.x, y: targetGapped.y }];
        const targetSource = [{ x: targetGapped.x, y: sourceGapped.y }];
        // this handles edges with same handle positions
        if (dirAccessor === 'x') {
            points = sourceDir.x === currDir ? targetSource : sourceTarget;
        }
        else {
            points = sourceDir.y === currDir ? sourceTarget : targetSource;
        }
        if (sourcePosition === targetPosition) {
            const diff = Math.abs(source[dirAccessor] - target[dirAccessor]);
            // if an edge goes from right to right for example (sourcePosition === targetPosition) and the distance between source.x and target.x is less than the offset, the added point and the gapped source/target will overlap. This leads to a weird edge path. To avoid this we add a gapOffset to the source/target
            if (diff <= offset) {
                const gapOffset = Math.min(offset - 1, offset - diff);
                if (sourceDir[dirAccessor] === currDir) {
                    sourceGapOffset[dirAccessor] = (sourceGapped[dirAccessor] > source[dirAccessor] ? -1 : 1) * gapOffset;
                }
                else {
                    targetGapOffset[dirAccessor] = (targetGapped[dirAccessor] > target[dirAccessor] ? -1 : 1) * gapOffset;
                }
            }
        }
        // these are conditions for handling mixed handle positions like Right -> Bottom for example
        if (sourcePosition !== targetPosition) {
            const dirAccessorOpposite = dirAccessor === 'x' ? 'y' : 'x';
            const isSameDir = sourceDir[dirAccessor] === targetDir[dirAccessorOpposite];
            const sourceGtTargetOppo = sourceGapped[dirAccessorOpposite] > targetGapped[dirAccessorOpposite];
            const sourceLtTargetOppo = sourceGapped[dirAccessorOpposite] < targetGapped[dirAccessorOpposite];
            const flipSourceTarget = (sourceDir[dirAccessor] === 1 && ((!isSameDir && sourceGtTargetOppo) || (isSameDir && sourceLtTargetOppo))) ||
                (sourceDir[dirAccessor] !== 1 && ((!isSameDir && sourceLtTargetOppo) || (isSameDir && sourceGtTargetOppo)));
            if (flipSourceTarget) {
                points = dirAccessor === 'x' ? sourceTarget : targetSource;
            }
        }
        const sourceGapPoint = { x: sourceGapped.x + sourceGapOffset.x, y: sourceGapped.y + sourceGapOffset.y };
        const targetGapPoint = { x: targetGapped.x + targetGapOffset.x, y: targetGapped.y + targetGapOffset.y };
        const maxXDistance = Math.max(Math.abs(sourceGapPoint.x - points[0].x), Math.abs(targetGapPoint.x - points[0].x));
        const maxYDistance = Math.max(Math.abs(sourceGapPoint.y - points[0].y), Math.abs(targetGapPoint.y - points[0].y));
        // we want to place the label on the longest segment of the edge
        if (maxXDistance >= maxYDistance) {
            centerX = (sourceGapPoint.x + targetGapPoint.x) / 2;
            centerY = points[0].y;
        }
        else {
            centerX = points[0].x;
            centerY = (sourceGapPoint.y + targetGapPoint.y) / 2;
        }
    }
    const pathPoints = [
        source,
        { x: sourceGapped.x + sourceGapOffset.x, y: sourceGapped.y + sourceGapOffset.y },
        ...points,
        { x: targetGapped.x + targetGapOffset.x, y: targetGapped.y + targetGapOffset.y },
        target,
    ];
    return [pathPoints, centerX, centerY, defaultOffsetX, defaultOffsetY];
}
function getBend(a, b, c, size) {
    const bendSize = Math.min(distance(a, b) / 2, distance(b, c) / 2, size);
    const { x, y } = b;
    // no bend
    if ((a.x === x && x === c.x) || (a.y === y && y === c.y)) {
        return `L${x} ${y}`;
    }
    // first segment is horizontal
    if (a.y === y) {
        const xDir = a.x < c.x ? -1 : 1;
        const yDir = a.y < c.y ? 1 : -1;
        return `L ${x + bendSize * xDir},${y}Q ${x},${y} ${x},${y + bendSize * yDir}`;
    }
    const xDir = a.x < c.x ? 1 : -1;
    const yDir = a.y < c.y ? -1 : 1;
    return `L ${x},${y + bendSize * yDir}Q ${x},${y} ${x + bendSize * xDir},${y}`;
}
/**
 * The `getSmoothStepPath` util returns everything you need to render a stepped path
 * between two nodes. The `borderRadius` property can be used to choose how rounded
 * the corners of those steps are.
 * @public
 * @returns A path string you can use in an SVG, the `labelX` and `labelY` position (center of path)
 * and `offsetX`, `offsetY` between source handle and label.
 *
 * - `path`: the path to use in an SVG `<path>` element.
 * - `labelX`: the `x` position you can use to render a label for this edge.
 * - `labelY`: the `y` position you can use to render a label for this edge.
 * - `offsetX`: the absolute difference between the source `x` position and the `x` position of the
 * middle of this path.
 * - `offsetY`: the absolute difference between the source `y` position and the `y` position of the
 * middle of this path.
 * @example
 * ```js
 *  const source = { x: 0, y: 20 };
 *  const target = { x: 150, y: 100 };
 *
 *  const [path, labelX, labelY, offsetX, offsetY] = getSmoothStepPath({
 *    sourceX: source.x,
 *    sourceY: source.y,
 *    sourcePosition: Position.Right,
 *    targetX: target.x,
 *    targetY: target.y,
 *    targetPosition: Position.Left,
 *  });
 * ```
 * @remarks This function returns a tuple (aka a fixed-size array) to make it easier to work with multiple edge paths at once.
 */
function getSmoothStepPath({ sourceX, sourceY, sourcePosition = Position.Bottom, targetX, targetY, targetPosition = Position.Top, borderRadius = 5, centerX, centerY, offset = 20, stepPosition = 0.5, }) {
    const [points, labelX, labelY, offsetX, offsetY] = getPoints({
        source: { x: sourceX, y: sourceY },
        sourcePosition,
        target: { x: targetX, y: targetY },
        targetPosition,
        center: { x: centerX, y: centerY },
        offset,
        stepPosition,
    });
    const path = points.reduce((res, p, i) => {
        let segment = '';
        if (i > 0 && i < points.length - 1) {
            segment = getBend(points[i - 1], p, points[i + 1], borderRadius);
        }
        else {
            segment = `${i === 0 ? 'M' : 'L'}${p.x} ${p.y}`;
        }
        res += segment;
        return res;
    }, '');
    return [path, labelX, labelY, offsetX, offsetY];
}

function isNodeInitialized(node) {
    return (node &&
        !!(node.internals.handleBounds || node.handles?.length) &&
        !!(node.measured.width || node.width || node.initialWidth));
}
function getEdgePosition(params) {
    const { sourceNode, targetNode } = params;
    if (!isNodeInitialized(sourceNode) || !isNodeInitialized(targetNode)) {
        return null;
    }
    const sourceHandleBounds = sourceNode.internals.handleBounds || toHandleBounds(sourceNode.handles);
    const targetHandleBounds = targetNode.internals.handleBounds || toHandleBounds(targetNode.handles);
    const sourceHandle = getHandle$1(sourceHandleBounds?.source ?? [], params.sourceHandle);
    const targetHandle = getHandle$1(
    // when connection type is loose we can define all handles as sources and connect source -> source
    params.connectionMode === ConnectionMode.Strict
        ? targetHandleBounds?.target ?? []
        : (targetHandleBounds?.target ?? []).concat(targetHandleBounds?.source ?? []), params.targetHandle);
    if (!sourceHandle || !targetHandle) {
        params.onError?.('008', errorMessages['error008'](!sourceHandle ? 'source' : 'target', {
            id: params.id,
            sourceHandle: params.sourceHandle,
            targetHandle: params.targetHandle,
        }));
        return null;
    }
    const sourcePosition = sourceHandle?.position || Position.Bottom;
    const targetPosition = targetHandle?.position || Position.Top;
    const source = getHandlePosition(sourceNode, sourceHandle, sourcePosition);
    const target = getHandlePosition(targetNode, targetHandle, targetPosition);
    return {
        sourceX: source.x,
        sourceY: source.y,
        targetX: target.x,
        targetY: target.y,
        sourcePosition,
        targetPosition,
    };
}
function toHandleBounds(handles) {
    if (!handles) {
        return null;
    }
    const source = [];
    const target = [];
    for (const handle of handles) {
        handle.width = handle.width ?? 1;
        handle.height = handle.height ?? 1;
        if (handle.type === 'source') {
            source.push(handle);
        }
        else if (handle.type === 'target') {
            target.push(handle);
        }
    }
    return {
        source,
        target,
    };
}
function getHandlePosition(node, handle, fallbackPosition = Position.Left, center = false) {
    const x = (handle?.x ?? 0) + node.internals.positionAbsolute.x;
    const y = (handle?.y ?? 0) + node.internals.positionAbsolute.y;
    const { width, height } = handle ?? getNodeDimensions(node);
    if (center) {
        return { x: x + width / 2, y: y + height / 2 };
    }
    const position = handle?.position ?? fallbackPosition;
    switch (position) {
        case Position.Top:
            return { x: x + width / 2, y };
        case Position.Right:
            return { x: x + width, y: y + height / 2 };
        case Position.Bottom:
            return { x: x + width / 2, y: y + height };
        case Position.Left:
            return { x, y: y + height / 2 };
    }
}
function getHandle$1(bounds, handleId) {
    if (!bounds) {
        return null;
    }
    // if no handleId is given, we use the first handle, otherwise we check for the id
    return (!handleId ? bounds[0] : bounds.find((d) => d.id === handleId)) || null;
}

function getMarkerId(marker, id) {
    if (!marker) {
        return '';
    }
    if (typeof marker === 'string') {
        return marker;
    }
    const idPrefix = id ? `${id}__` : '';
    return `${idPrefix}${Object.keys(marker)
        .sort()
        .map((key) => `${key}=${marker[key]}`)
        .join('&')}`;
}
function createMarkerIds(edges, { id, defaultColor, defaultMarkerStart, defaultMarkerEnd, }) {
    const ids = new Set();
    return edges
        .reduce((markers, edge) => {
        [edge.markerStart || defaultMarkerStart, edge.markerEnd || defaultMarkerEnd].forEach((marker) => {
            if (marker && typeof marker === 'object') {
                const markerId = getMarkerId(marker, id);
                if (!ids.has(markerId)) {
                    markers.push({ id: markerId, color: marker.color || defaultColor, ...marker });
                    ids.add(markerId);
                }
            }
        });
        return markers;
    }, [])
        .sort((a, b) => a.id.localeCompare(b.id));
}

const defaultOptions = {
    nodeOrigin: [0, 0],
    nodeExtent: infiniteExtent,
    elevateNodesOnSelect: true,
    defaults: {},
};
const adoptUserNodesDefaultOptions = {
    ...defaultOptions,
    checkEquality: true,
};
function mergeObjects(base, incoming) {
    const result = { ...base };
    for (const key in incoming) {
        if (incoming[key] !== undefined) {
            // typecast is safe here, because we check for undefined
            result[key] = incoming[key];
        }
    }
    return result;
}
function updateAbsolutePositions(nodeLookup, parentLookup, options) {
    const _options = mergeObjects(defaultOptions, options);
    for (const node of nodeLookup.values()) {
        if (node.parentId) {
            updateChildNode(node, nodeLookup, parentLookup, _options);
        }
        else {
            const positionWithOrigin = getNodePositionWithOrigin(node, _options.nodeOrigin);
            const extent = isCoordinateExtent(node.extent) ? node.extent : _options.nodeExtent;
            const clampedPosition = clampPosition(positionWithOrigin, extent, getNodeDimensions(node));
            node.internals.positionAbsolute = clampedPosition;
        }
    }
}
function adoptUserNodes(nodes, nodeLookup, parentLookup, options) {
    const _options = mergeObjects(adoptUserNodesDefaultOptions, options);
    let nodesInitialized = nodes.length > 0;
    const tmpLookup = new Map(nodeLookup);
    const selectedNodeZ = _options?.elevateNodesOnSelect ? 1000 : 0;
    nodeLookup.clear();
    parentLookup.clear();
    for (const userNode of nodes) {
        let internalNode = tmpLookup.get(userNode.id);
        if (_options.checkEquality && userNode === internalNode?.internals.userNode) {
            nodeLookup.set(userNode.id, internalNode);
        }
        else {
            const positionWithOrigin = getNodePositionWithOrigin(userNode, _options.nodeOrigin);
            const extent = isCoordinateExtent(userNode.extent) ? userNode.extent : _options.nodeExtent;
            const clampedPosition = clampPosition(positionWithOrigin, extent, getNodeDimensions(userNode));
            internalNode = {
                ..._options.defaults,
                ...userNode,
                measured: {
                    width: userNode.measured?.width,
                    height: userNode.measured?.height,
                },
                internals: {
                    positionAbsolute: clampedPosition,
                    // if user re-initializes the node or removes `measured` for whatever reason, we reset the handleBounds so that the node gets re-measured
                    handleBounds: !userNode.measured ? undefined : internalNode?.internals.handleBounds,
                    z: calculateZ(userNode, selectedNodeZ),
                    userNode,
                },
            };
            nodeLookup.set(userNode.id, internalNode);
        }
        if ((internalNode.measured === undefined ||
            internalNode.measured.width === undefined ||
            internalNode.measured.height === undefined) &&
            !internalNode.hidden) {
            nodesInitialized = false;
        }
        if (userNode.parentId) {
            updateChildNode(internalNode, nodeLookup, parentLookup, options);
        }
    }
    return nodesInitialized;
}
function updateParentLookup(node, parentLookup) {
    if (!node.parentId) {
        return;
    }
    const childNodes = parentLookup.get(node.parentId);
    if (childNodes) {
        childNodes.set(node.id, node);
    }
    else {
        parentLookup.set(node.parentId, new Map([[node.id, node]]));
    }
}
/**
 * Updates positionAbsolute and zIndex of a child node and the parentLookup.
 */
function updateChildNode(node, nodeLookup, parentLookup, options) {
    const { elevateNodesOnSelect, nodeOrigin, nodeExtent } = mergeObjects(defaultOptions, options);
    const parentId = node.parentId;
    const parentNode = nodeLookup.get(parentId);
    if (!parentNode) {
        console.warn(`Parent node ${parentId} not found. Please make sure that parent nodes are in front of their child nodes in the nodes array.`);
        return;
    }
    updateParentLookup(node, parentLookup);
    const selectedNodeZ = elevateNodesOnSelect ? 1000 : 0;
    const { x, y, z } = calculateChildXYZ(node, parentNode, nodeOrigin, nodeExtent, selectedNodeZ);
    const { positionAbsolute } = node.internals;
    const positionChanged = x !== positionAbsolute.x || y !== positionAbsolute.y;
    if (positionChanged || z !== node.internals.z) {
        // we create a new object to mark the node as updated
        nodeLookup.set(node.id, {
            ...node,
            internals: {
                ...node.internals,
                positionAbsolute: positionChanged ? { x, y } : positionAbsolute,
                z,
            },
        });
    }
}
function calculateZ(node, selectedNodeZ) {
    return (isNumeric(node.zIndex) ? node.zIndex : 0) + (node.selected ? selectedNodeZ : 0);
}
function calculateChildXYZ(childNode, parentNode, nodeOrigin, nodeExtent, selectedNodeZ) {
    const { x: parentX, y: parentY } = parentNode.internals.positionAbsolute;
    const childDimensions = getNodeDimensions(childNode);
    const positionWithOrigin = getNodePositionWithOrigin(childNode, nodeOrigin);
    const clampedPosition = isCoordinateExtent(childNode.extent)
        ? clampPosition(positionWithOrigin, childNode.extent, childDimensions)
        : positionWithOrigin;
    let absolutePosition = clampPosition({ x: parentX + clampedPosition.x, y: parentY + clampedPosition.y }, nodeExtent, childDimensions);
    if (childNode.extent === 'parent') {
        absolutePosition = clampPositionToParent(absolutePosition, childDimensions, parentNode);
    }
    const childZ = calculateZ(childNode, selectedNodeZ);
    const parentZ = parentNode.internals.z ?? 0;
    return {
        x: absolutePosition.x,
        y: absolutePosition.y,
        z: parentZ >= childZ ? parentZ + 1 : childZ,
    };
}
function handleExpandParent(children, nodeLookup, parentLookup, nodeOrigin = [0, 0]) {
    const changes = [];
    const parentExpansions = new Map();
    // determine the expanded rectangle the child nodes would take for each parent
    for (const child of children) {
        const parent = nodeLookup.get(child.parentId);
        if (!parent) {
            continue;
        }
        const parentRect = parentExpansions.get(child.parentId)?.expandedRect ?? nodeToRect(parent);
        const expandedRect = getBoundsOfRects(parentRect, child.rect);
        parentExpansions.set(child.parentId, { expandedRect, parent });
    }
    if (parentExpansions.size > 0) {
        parentExpansions.forEach(({ expandedRect, parent }, parentId) => {
            // determine the position & dimensions of the parent
            const positionAbsolute = parent.internals.positionAbsolute;
            const dimensions = getNodeDimensions(parent);
            const origin = parent.origin ?? nodeOrigin;
            // determine how much the parent expands in width and position
            const xChange = expandedRect.x < positionAbsolute.x ? Math.round(Math.abs(positionAbsolute.x - expandedRect.x)) : 0;
            const yChange = expandedRect.y < positionAbsolute.y ? Math.round(Math.abs(positionAbsolute.y - expandedRect.y)) : 0;
            const newWidth = Math.max(dimensions.width, Math.round(expandedRect.width));
            const newHeight = Math.max(dimensions.height, Math.round(expandedRect.height));
            const widthChange = (newWidth - dimensions.width) * origin[0];
            const heightChange = (newHeight - dimensions.height) * origin[1];
            // We need to correct the position of the parent node if the origin is not [0,0]
            if (xChange > 0 || yChange > 0 || widthChange || heightChange) {
                changes.push({
                    id: parentId,
                    type: 'position',
                    position: {
                        x: parent.position.x - xChange + widthChange,
                        y: parent.position.y - yChange + heightChange,
                    },
                });
                /*
                 * We move all child nodes in the oppsite direction
                 * so the x,y changes of the parent do not move the children
                 */
                parentLookup.get(parentId)?.forEach((childNode) => {
                    if (!children.some((child) => child.id === childNode.id)) {
                        changes.push({
                            id: childNode.id,
                            type: 'position',
                            position: {
                                x: childNode.position.x + xChange,
                                y: childNode.position.y + yChange,
                            },
                        });
                    }
                });
            }
            // We need to correct the dimensions of the parent node if the origin is not [0,0]
            if (dimensions.width < expandedRect.width || dimensions.height < expandedRect.height || xChange || yChange) {
                changes.push({
                    id: parentId,
                    type: 'dimensions',
                    setAttributes: true,
                    dimensions: {
                        width: newWidth + (xChange ? origin[0] * xChange - widthChange : 0),
                        height: newHeight + (yChange ? origin[1] * yChange - heightChange : 0),
                    },
                });
            }
        });
    }
    return changes;
}
function updateNodeInternals(updates, nodeLookup, parentLookup, domNode, nodeOrigin, nodeExtent) {
    const viewportNode = domNode?.querySelector('.xyflow__viewport');
    let updatedInternals = false;
    if (!viewportNode) {
        return { changes: [], updatedInternals };
    }
    const changes = [];
    const style = window.getComputedStyle(viewportNode);
    const { m22: zoom } = new window.DOMMatrixReadOnly(style.transform);
    // in this array we collect nodes, that might trigger changes (like expanding parent)
    const parentExpandChildren = [];
    for (const update of updates.values()) {
        const node = nodeLookup.get(update.id);
        if (!node) {
            continue;
        }
        if (node.hidden) {
            nodeLookup.set(node.id, {
                ...node,
                internals: {
                    ...node.internals,
                    handleBounds: undefined,
                },
            });
            updatedInternals = true;
            continue;
        }
        const dimensions = getDimensions(update.nodeElement);
        const dimensionChanged = node.measured.width !== dimensions.width || node.measured.height !== dimensions.height;
        const doUpdate = !!(dimensions.width &&
            dimensions.height &&
            (dimensionChanged || !node.internals.handleBounds || update.force));
        if (doUpdate) {
            const nodeBounds = update.nodeElement.getBoundingClientRect();
            const extent = isCoordinateExtent(node.extent) ? node.extent : nodeExtent;
            let { positionAbsolute } = node.internals;
            if (node.parentId && node.extent === 'parent') {
                positionAbsolute = clampPositionToParent(positionAbsolute, dimensions, nodeLookup.get(node.parentId));
            }
            else if (extent) {
                positionAbsolute = clampPosition(positionAbsolute, extent, dimensions);
            }
            const newNode = {
                ...node,
                measured: dimensions,
                internals: {
                    ...node.internals,
                    positionAbsolute,
                    handleBounds: {
                        source: getHandleBounds('source', update.nodeElement, nodeBounds, zoom, node.id),
                        target: getHandleBounds('target', update.nodeElement, nodeBounds, zoom, node.id),
                    },
                },
            };
            nodeLookup.set(node.id, newNode);
            if (node.parentId) {
                updateChildNode(newNode, nodeLookup, parentLookup, { nodeOrigin });
            }
            updatedInternals = true;
            if (dimensionChanged) {
                changes.push({
                    id: node.id,
                    type: 'dimensions',
                    dimensions,
                });
                if (node.expandParent && node.parentId) {
                    parentExpandChildren.push({
                        id: node.id,
                        parentId: node.parentId,
                        rect: nodeToRect(newNode, nodeOrigin),
                    });
                }
            }
        }
    }
    if (parentExpandChildren.length > 0) {
        const parentExpandChanges = handleExpandParent(parentExpandChildren, nodeLookup, parentLookup, nodeOrigin);
        changes.push(...parentExpandChanges);
    }
    return { changes, updatedInternals };
}
async function panBy({ delta, panZoom, transform, translateExtent, width, height, }) {
    if (!panZoom || (!delta.x && !delta.y)) {
        return Promise.resolve(false);
    }
    const nextViewport = await panZoom.setViewportConstrained({
        x: transform[0] + delta.x,
        y: transform[1] + delta.y,
        zoom: transform[2],
    }, [
        [0, 0],
        [width, height],
    ], translateExtent);
    const transformChanged = !!nextViewport &&
        (nextViewport.x !== transform[0] || nextViewport.y !== transform[1] || nextViewport.k !== transform[2]);
    return Promise.resolve(transformChanged);
}
/**
 * this function adds the connection to the connectionLookup
 * at the following keys: nodeId-type-handleId, nodeId-type and nodeId
 * @param type type of the connection
 * @param connection connection that should be added to the lookup
 * @param connectionKey at which key the connection should be added
 * @param connectionLookup reference to the connection lookup
 * @param nodeId nodeId of the connection
 * @param handleId handleId of the conneciton
 */
function addConnectionToLookup(type, connection, connectionKey, connectionLookup, nodeId, handleId) {
    /*
     * We add the connection to the connectionLookup at the following keys
     * 1. nodeId, 2. nodeId-type, 3. nodeId-type-handleId
     * If the key already exists, we add the connection to the existing map
     */
    let key = nodeId;
    const nodeMap = connectionLookup.get(key) || new Map();
    connectionLookup.set(key, nodeMap.set(connectionKey, connection));
    key = `${nodeId}-${type}`;
    const typeMap = connectionLookup.get(key) || new Map();
    connectionLookup.set(key, typeMap.set(connectionKey, connection));
    if (handleId) {
        key = `${nodeId}-${type}-${handleId}`;
        const handleMap = connectionLookup.get(key) || new Map();
        connectionLookup.set(key, handleMap.set(connectionKey, connection));
    }
}
function updateConnectionLookup(connectionLookup, edgeLookup, edges) {
    connectionLookup.clear();
    edgeLookup.clear();
    for (const edge of edges) {
        const { source: sourceNode, target: targetNode, sourceHandle = null, targetHandle = null } = edge;
        const connection = { edgeId: edge.id, source: sourceNode, target: targetNode, sourceHandle, targetHandle };
        const sourceKey = `${sourceNode}-${sourceHandle}--${targetNode}-${targetHandle}`;
        const targetKey = `${targetNode}-${targetHandle}--${sourceNode}-${sourceHandle}`;
        addConnectionToLookup('source', connection, targetKey, connectionLookup, sourceNode, sourceHandle);
        addConnectionToLookup('target', connection, sourceKey, connectionLookup, targetNode, targetHandle);
        edgeLookup.set(edge.id, edge);
    }
}

function isParentSelected(node, nodeLookup) {
    if (!node.parentId) {
        return false;
    }
    const parentNode = nodeLookup.get(node.parentId);
    if (!parentNode) {
        return false;
    }
    if (parentNode.selected) {
        return true;
    }
    return isParentSelected(parentNode, nodeLookup);
}
function hasSelector(target, selector, domNode) {
    let current = target;
    do {
        if (current?.matches?.(selector))
            return true;
        if (current === domNode)
            return false;
        current = current?.parentElement;
    } while (current);
    return false;
}
// looks for all selected nodes and created a NodeDragItem for each of them
function getDragItems(nodeLookup, nodesDraggable, mousePos, nodeId) {
    const dragItems = new Map();
    for (const [id, node] of nodeLookup) {
        if ((node.selected || node.id === nodeId) &&
            (!node.parentId || !isParentSelected(node, nodeLookup)) &&
            (node.draggable || (nodesDraggable && typeof node.draggable === 'undefined'))) {
            const internalNode = nodeLookup.get(id);
            if (internalNode) {
                dragItems.set(id, {
                    id,
                    position: internalNode.position || { x: 0, y: 0 },
                    distance: {
                        x: mousePos.x - internalNode.internals.positionAbsolute.x,
                        y: mousePos.y - internalNode.internals.positionAbsolute.y,
                    },
                    extent: internalNode.extent,
                    parentId: internalNode.parentId,
                    origin: internalNode.origin,
                    expandParent: internalNode.expandParent,
                    internals: {
                        positionAbsolute: internalNode.internals.positionAbsolute || { x: 0, y: 0 },
                    },
                    measured: {
                        width: internalNode.measured.width ?? 0,
                        height: internalNode.measured.height ?? 0,
                    },
                });
            }
        }
    }
    return dragItems;
}
/*
 * returns two params:
 * 1. the dragged node (or the first of the list, if we are dragging a node selection)
 * 2. array of selected nodes (for multi selections)
 */
function getEventHandlerParams({ nodeId, dragItems, nodeLookup, dragging = true, }) {
    const nodesFromDragItems = [];
    for (const [id, dragItem] of dragItems) {
        const node = nodeLookup.get(id)?.internals.userNode;
        if (node) {
            nodesFromDragItems.push({
                ...node,
                position: dragItem.position,
                dragging,
            });
        }
    }
    if (!nodeId) {
        return [nodesFromDragItems[0], nodesFromDragItems];
    }
    const node = nodeLookup.get(nodeId)?.internals.userNode;
    return [
        !node
            ? nodesFromDragItems[0]
            : {
                ...node,
                position: dragItems.get(nodeId)?.position || node.position,
                dragging,
            },
        nodesFromDragItems,
    ];
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function XYDrag({ onNodeMouseDown, getStoreItems, onDragStart, onDrag, onDragStop, }) {
    let lastPos = { x: null, y: null };
    let autoPanId = 0;
    let dragItems = new Map();
    let autoPanStarted = false;
    let mousePosition = { x: 0, y: 0 };
    let containerBounds = null;
    let dragStarted = false;
    let d3Selection = null;
    let abortDrag = false; // prevents unintentional dragging on multitouch
    let nodePositionsChanged = false;
    // public functions
    function update({ noDragClassName, handleSelector, domNode, isSelectable, nodeId, nodeClickDistance = 0, }) {
        d3Selection = select(domNode);
        function updateNodes({ x, y }, dragEvent) {
            const { nodeLookup, nodeExtent, snapGrid, snapToGrid, nodeOrigin, onNodeDrag, onSelectionDrag, onError, updateNodePositions, } = getStoreItems();
            lastPos = { x, y };
            let hasChange = false;
            let nodesBox = { x: 0, y: 0, x2: 0, y2: 0 };
            if (dragItems.size > 1 && nodeExtent) {
                const rect = getInternalNodesBounds(dragItems);
                nodesBox = rectToBox(rect);
            }
            for (const [id, dragItem] of dragItems) {
                if (!nodeLookup.has(id)) {
                    /*
                     * if the node is not in the nodeLookup anymore, it was probably deleted while dragging
                     * and we don't need to update it anymore
                     */
                    continue;
                }
                let nextPosition = { x: x - dragItem.distance.x, y: y - dragItem.distance.y };
                if (snapToGrid) {
                    nextPosition = snapPosition(nextPosition, snapGrid);
                }
                /*
                 * if there is selection with multiple nodes and a node extent is set, we need to adjust the node extent for each node
                 * based on its position so that the node stays at it's position relative to the selection.
                 */
                let adjustedNodeExtent = [
                    [nodeExtent[0][0], nodeExtent[0][1]],
                    [nodeExtent[1][0], nodeExtent[1][1]],
                ];
                if (dragItems.size > 1 && nodeExtent && !dragItem.extent) {
                    const { positionAbsolute } = dragItem.internals;
                    const x1 = positionAbsolute.x - nodesBox.x + nodeExtent[0][0];
                    const x2 = positionAbsolute.x + dragItem.measured.width - nodesBox.x2 + nodeExtent[1][0];
                    const y1 = positionAbsolute.y - nodesBox.y + nodeExtent[0][1];
                    const y2 = positionAbsolute.y + dragItem.measured.height - nodesBox.y2 + nodeExtent[1][1];
                    adjustedNodeExtent = [
                        [x1, y1],
                        [x2, y2],
                    ];
                }
                const { position, positionAbsolute } = calculateNodePosition({
                    nodeId: id,
                    nextPosition,
                    nodeLookup,
                    nodeExtent: adjustedNodeExtent,
                    nodeOrigin,
                    onError,
                });
                // we want to make sure that we only fire a change event when there is a change
                hasChange = hasChange || dragItem.position.x !== position.x || dragItem.position.y !== position.y;
                dragItem.position = position;
                dragItem.internals.positionAbsolute = positionAbsolute;
            }
            nodePositionsChanged = nodePositionsChanged || hasChange;
            if (!hasChange) {
                return;
            }
            updateNodePositions(dragItems, true);
            if (dragEvent && (onDrag || onNodeDrag || (!nodeId && onSelectionDrag))) {
                const [currentNode, currentNodes] = getEventHandlerParams({
                    nodeId,
                    dragItems,
                    nodeLookup,
                });
                onDrag?.(dragEvent, dragItems, currentNode, currentNodes);
                onNodeDrag?.(dragEvent, currentNode, currentNodes);
                if (!nodeId) {
                    onSelectionDrag?.(dragEvent, currentNodes);
                }
            }
        }
        async function autoPan() {
            if (!containerBounds) {
                return;
            }
            const { transform, panBy, autoPanSpeed, autoPanOnNodeDrag } = getStoreItems();
            if (!autoPanOnNodeDrag) {
                autoPanStarted = false;
                cancelAnimationFrame(autoPanId);
                return;
            }
            const [xMovement, yMovement] = calcAutoPan(mousePosition, containerBounds, autoPanSpeed);
            if (xMovement !== 0 || yMovement !== 0) {
                lastPos.x = (lastPos.x ?? 0) - xMovement / transform[2];
                lastPos.y = (lastPos.y ?? 0) - yMovement / transform[2];
                if (await panBy({ x: xMovement, y: yMovement })) {
                    updateNodes(lastPos, null);
                }
            }
            autoPanId = requestAnimationFrame(autoPan);
        }
        function startDrag(event) {
            const { nodeLookup, multiSelectionActive, nodesDraggable, transform, snapGrid, snapToGrid, selectNodesOnDrag, onNodeDragStart, onSelectionDragStart, unselectNodesAndEdges, } = getStoreItems();
            dragStarted = true;
            if ((!selectNodesOnDrag || !isSelectable) && !multiSelectionActive && nodeId) {
                if (!nodeLookup.get(nodeId)?.selected) {
                    // we need to reset selected nodes when selectNodesOnDrag=false
                    unselectNodesAndEdges();
                }
            }
            if (isSelectable && selectNodesOnDrag && nodeId) {
                onNodeMouseDown?.(nodeId);
            }
            const pointerPos = getPointerPosition(event.sourceEvent, { transform, snapGrid, snapToGrid, containerBounds });
            lastPos = pointerPos;
            dragItems = getDragItems(nodeLookup, nodesDraggable, pointerPos, nodeId);
            if (dragItems.size > 0 && (onDragStart || onNodeDragStart || (!nodeId && onSelectionDragStart))) {
                const [currentNode, currentNodes] = getEventHandlerParams({
                    nodeId,
                    dragItems,
                    nodeLookup,
                });
                onDragStart?.(event.sourceEvent, dragItems, currentNode, currentNodes);
                onNodeDragStart?.(event.sourceEvent, currentNode, currentNodes);
                if (!nodeId) {
                    onSelectionDragStart?.(event.sourceEvent, currentNodes);
                }
            }
        }
        const d3DragInstance = drag()
            .clickDistance(nodeClickDistance)
            .on('start', (event) => {
            const { domNode, nodeDragThreshold, transform, snapGrid, snapToGrid } = getStoreItems();
            containerBounds = domNode?.getBoundingClientRect() || null;
            abortDrag = false;
            nodePositionsChanged = false;
            if (nodeDragThreshold === 0) {
                startDrag(event);
            }
            const pointerPos = getPointerPosition(event.sourceEvent, { transform, snapGrid, snapToGrid, containerBounds });
            lastPos = pointerPos;
            mousePosition = getEventPosition(event.sourceEvent, containerBounds);
        })
            .on('drag', (event) => {
            const { autoPanOnNodeDrag, transform, snapGrid, snapToGrid, nodeDragThreshold, nodeLookup } = getStoreItems();
            const pointerPos = getPointerPosition(event.sourceEvent, { transform, snapGrid, snapToGrid, containerBounds });
            if ((event.sourceEvent.type === 'touchmove' && event.sourceEvent.touches.length > 1) ||
                // if user deletes a node while dragging, we need to abort the drag to prevent errors
                (nodeId && !nodeLookup.has(nodeId))) {
                abortDrag = true;
            }
            if (abortDrag) {
                return;
            }
            if (!autoPanStarted && autoPanOnNodeDrag && dragStarted) {
                autoPanStarted = true;
                autoPan();
            }
            if (!dragStarted) {
                const x = pointerPos.xSnapped - (lastPos.x ?? 0);
                const y = pointerPos.ySnapped - (lastPos.y ?? 0);
                const distance = Math.sqrt(x * x + y * y);
                if (distance > nodeDragThreshold) {
                    startDrag(event);
                }
            }
            // skip events without movement
            if ((lastPos.x !== pointerPos.xSnapped || lastPos.y !== pointerPos.ySnapped) && dragItems && dragStarted) {
                // dragEvent = event.sourceEvent as MouseEvent;
                mousePosition = getEventPosition(event.sourceEvent, containerBounds);
                updateNodes(pointerPos, event.sourceEvent);
            }
        })
            .on('end', (event) => {
            if (!dragStarted || abortDrag) {
                return;
            }
            autoPanStarted = false;
            dragStarted = false;
            cancelAnimationFrame(autoPanId);
            if (dragItems.size > 0) {
                const { nodeLookup, updateNodePositions, onNodeDragStop, onSelectionDragStop } = getStoreItems();
                if (nodePositionsChanged) {
                    updateNodePositions(dragItems, false);
                    nodePositionsChanged = false;
                }
                if (onDragStop || onNodeDragStop || (!nodeId && onSelectionDragStop)) {
                    const [currentNode, currentNodes] = getEventHandlerParams({
                        nodeId,
                        dragItems,
                        nodeLookup,
                        dragging: false,
                    });
                    onDragStop?.(event.sourceEvent, dragItems, currentNode, currentNodes);
                    onNodeDragStop?.(event.sourceEvent, currentNode, currentNodes);
                    if (!nodeId) {
                        onSelectionDragStop?.(event.sourceEvent, currentNodes);
                    }
                }
            }
        })
            .filter((event) => {
            const target = event.target;
            const isDraggable = !event.button &&
                (!noDragClassName || !hasSelector(target, `.${noDragClassName}`, domNode)) &&
                (!handleSelector || hasSelector(target, handleSelector, domNode));
            return isDraggable;
        });
        d3Selection.call(d3DragInstance);
    }
    function destroy() {
        d3Selection?.on('.drag', null);
    }
    return {
        update,
        destroy,
    };
}

function getNodesWithinDistance(position, nodeLookup, distance) {
    const nodes = [];
    const rect = {
        x: position.x - distance,
        y: position.y - distance,
        width: distance * 2,
        height: distance * 2,
    };
    for (const node of nodeLookup.values()) {
        if (getOverlappingArea(rect, nodeToRect(node)) > 0) {
            nodes.push(node);
        }
    }
    return nodes;
}
/*
 * this distance is used for the area around the user pointer
 * while doing a connection for finding the closest nodes
 */
const ADDITIONAL_DISTANCE = 250;
function getClosestHandle(position, connectionRadius, nodeLookup, fromHandle) {
    let closestHandles = [];
    let minDistance = Infinity;
    const closeNodes = getNodesWithinDistance(position, nodeLookup, connectionRadius + ADDITIONAL_DISTANCE);
    for (const node of closeNodes) {
        const allHandles = [...(node.internals.handleBounds?.source ?? []), ...(node.internals.handleBounds?.target ?? [])];
        for (const handle of allHandles) {
            // if the handle is the same as the fromHandle we skip it
            if (fromHandle.nodeId === handle.nodeId && fromHandle.type === handle.type && fromHandle.id === handle.id) {
                continue;
            }
            // determine absolute position of the handle
            const { x, y } = getHandlePosition(node, handle, handle.position, true);
            const distance = Math.sqrt(Math.pow(x - position.x, 2) + Math.pow(y - position.y, 2));
            if (distance > connectionRadius) {
                continue;
            }
            if (distance < minDistance) {
                closestHandles = [{ ...handle, x, y }];
                minDistance = distance;
            }
            else if (distance === minDistance) {
                // when multiple handles are on the same distance we collect all of them
                closestHandles.push({ ...handle, x, y });
            }
        }
    }
    if (!closestHandles.length) {
        return null;
    }
    // when multiple handles overlay each other we prefer the opposite handle
    if (closestHandles.length > 1) {
        const oppositeHandleType = fromHandle.type === 'source' ? 'target' : 'source';
        return closestHandles.find((handle) => handle.type === oppositeHandleType) ?? closestHandles[0];
    }
    return closestHandles[0];
}
function getHandle(nodeId, handleType, handleId, nodeLookup, connectionMode, withAbsolutePosition = false) {
    const node = nodeLookup.get(nodeId);
    if (!node) {
        return null;
    }
    const handles = connectionMode === 'strict'
        ? node.internals.handleBounds?.[handleType]
        : [...(node.internals.handleBounds?.source ?? []), ...(node.internals.handleBounds?.target ?? [])];
    const handle = (handleId ? handles?.find((h) => h.id === handleId) : handles?.[0]) ?? null;
    return handle && withAbsolutePosition
        ? { ...handle, ...getHandlePosition(node, handle, handle.position, true) }
        : handle;
}
function getHandleType(edgeUpdaterType, handleDomNode) {
    if (edgeUpdaterType) {
        return edgeUpdaterType;
    }
    else if (handleDomNode?.classList.contains('target')) {
        return 'target';
    }
    else if (handleDomNode?.classList.contains('source')) {
        return 'source';
    }
    return null;
}
function isConnectionValid(isInsideConnectionRadius, isHandleValid) {
    let isValid = null;
    if (isHandleValid) {
        isValid = true;
    }
    else if (isInsideConnectionRadius && !isHandleValid) {
        isValid = false;
    }
    return isValid;
}

const alwaysValid = () => true;
function onPointerDown(event, { connectionMode, connectionRadius, handleId, nodeId, edgeUpdaterType, isTarget, domNode, nodeLookup, lib, autoPanOnConnect, flowId, panBy, cancelConnection, onConnectStart, onConnect, onConnectEnd, isValidConnection = alwaysValid, onReconnectEnd, updateConnection, getTransform, getFromHandle, autoPanSpeed, dragThreshold = 1, }) {
    // when xyflow is used inside a shadow root we can't use document
    const doc = getHostForElement(event.target);
    let autoPanId = 0;
    let closestHandle;
    const { x, y } = getEventPosition(event);
    const clickedHandle = doc?.elementFromPoint(x, y);
    const handleType = getHandleType(edgeUpdaterType, clickedHandle);
    const containerBounds = domNode?.getBoundingClientRect();
    let connectionStarted = false;
    if (!containerBounds || !handleType) {
        return;
    }
    const fromHandleInternal = getHandle(nodeId, handleType, handleId, nodeLookup, connectionMode);
    if (!fromHandleInternal) {
        return;
    }
    let position = getEventPosition(event, containerBounds);
    let autoPanStarted = false;
    let connection = null;
    let isValid = false;
    let handleDomNode = null;
    // when the user is moving the mouse close to the edge of the canvas while connecting we move the canvas
    function autoPan() {
        if (!autoPanOnConnect || !containerBounds) {
            return;
        }
        const [x, y] = calcAutoPan(position, containerBounds, autoPanSpeed);
        panBy({ x, y });
        autoPanId = requestAnimationFrame(autoPan);
    }
    // Stays the same for all consecutive pointermove events
    const fromHandle = {
        ...fromHandleInternal,
        nodeId,
        type: handleType,
        position: fromHandleInternal.position,
    };
    const fromNodeInternal = nodeLookup.get(nodeId);
    const from = getHandlePosition(fromNodeInternal, fromHandle, Position.Left, true);
    let previousConnection = {
        inProgress: true,
        isValid: null,
        from,
        fromHandle,
        fromPosition: fromHandle.position,
        fromNode: fromNodeInternal,
        to: position,
        toHandle: null,
        toPosition: oppositePosition[fromHandle.position],
        toNode: null,
    };
    function startConnection() {
        connectionStarted = true;
        updateConnection(previousConnection);
        onConnectStart?.(event, { nodeId, handleId, handleType });
    }
    if (dragThreshold === 0) {
        startConnection();
    }
    function onPointerMove(event) {
        if (!connectionStarted) {
            const { x: evtX, y: evtY } = getEventPosition(event);
            const dx = evtX - x;
            const dy = evtY - y;
            const nextConnectionStarted = dx * dx + dy * dy > dragThreshold * dragThreshold;
            if (!nextConnectionStarted) {
                return;
            }
            startConnection();
        }
        if (!getFromHandle() || !fromHandle) {
            onPointerUp(event);
            return;
        }
        const transform = getTransform();
        position = getEventPosition(event, containerBounds);
        closestHandle = getClosestHandle(pointToRendererPoint(position, transform, false, [1, 1]), connectionRadius, nodeLookup, fromHandle);
        if (!autoPanStarted) {
            autoPan();
            autoPanStarted = true;
        }
        const result = isValidHandle(event, {
            handle: closestHandle,
            connectionMode,
            fromNodeId: nodeId,
            fromHandleId: handleId,
            fromType: isTarget ? 'target' : 'source',
            isValidConnection,
            doc,
            lib,
            flowId,
            nodeLookup,
        });
        handleDomNode = result.handleDomNode;
        connection = result.connection;
        isValid = isConnectionValid(!!closestHandle, result.isValid);
        const newConnection = {
            // from stays the same
            ...previousConnection,
            isValid,
            to: result.toHandle && isValid
                ? rendererPointToPoint({ x: result.toHandle.x, y: result.toHandle.y }, transform)
                : position,
            toHandle: result.toHandle,
            toPosition: isValid && result.toHandle ? result.toHandle.position : oppositePosition[fromHandle.position],
            toNode: result.toHandle ? nodeLookup.get(result.toHandle.nodeId) : null,
        };
        /*
         * we don't want to trigger an update when the connection
         * is snapped to the same handle as before
         */
        if (isValid &&
            closestHandle &&
            previousConnection.toHandle &&
            newConnection.toHandle &&
            previousConnection.toHandle.type === newConnection.toHandle.type &&
            previousConnection.toHandle.nodeId === newConnection.toHandle.nodeId &&
            previousConnection.toHandle.id === newConnection.toHandle.id &&
            previousConnection.to.x === newConnection.to.x &&
            previousConnection.to.y === newConnection.to.y) {
            return;
        }
        updateConnection(newConnection);
        previousConnection = newConnection;
    }
    function onPointerUp(event) {
        if (connectionStarted) {
            if ((closestHandle || handleDomNode) && connection && isValid) {
                onConnect?.(connection);
            }
            /*
             * it's important to get a fresh reference from the store here
             * in order to get the latest state of onConnectEnd
             */
            // eslint-disable-next-line @typescript-eslint/no-unused-vars
            const { inProgress, ...connectionState } = previousConnection;
            const finalConnectionState = {
                ...connectionState,
                toPosition: previousConnection.toHandle ? previousConnection.toPosition : null,
            };
            onConnectEnd?.(event, finalConnectionState);
            if (edgeUpdaterType) {
                onReconnectEnd?.(event, finalConnectionState);
            }
        }
        cancelConnection();
        cancelAnimationFrame(autoPanId);
        autoPanStarted = false;
        isValid = false;
        connection = null;
        handleDomNode = null;
        doc.removeEventListener('mousemove', onPointerMove);
        doc.removeEventListener('mouseup', onPointerUp);
        doc.removeEventListener('touchmove', onPointerMove);
        doc.removeEventListener('touchend', onPointerUp);
    }
    doc.addEventListener('mousemove', onPointerMove);
    doc.addEventListener('mouseup', onPointerUp);
    doc.addEventListener('touchmove', onPointerMove);
    doc.addEventListener('touchend', onPointerUp);
}
// checks if  and returns connection in fom of an object { source: 123, target: 312 }
function isValidHandle(event, { handle, connectionMode, fromNodeId, fromHandleId, fromType, doc, lib, flowId, isValidConnection = alwaysValid, nodeLookup, }) {
    const isTarget = fromType === 'target';
    const handleDomNode = handle
        ? doc.querySelector(`.${lib}-flow__handle[data-id="${flowId}-${handle?.nodeId}-${handle?.id}-${handle?.type}"]`)
        : null;
    const { x, y } = getEventPosition(event);
    const handleBelow = doc.elementFromPoint(x, y);
    /*
     * we always want to prioritize the handle below the mouse cursor over the closest distance handle,
     * because it could be that the center of another handle is closer to the mouse pointer than the handle below the cursor
     */
    const handleToCheck = handleBelow?.classList.contains(`${lib}-flow__handle`) ? handleBelow : handleDomNode;
    const result = {
        handleDomNode: handleToCheck,
        isValid: false,
        connection: null,
        toHandle: null,
    };
    if (handleToCheck) {
        const handleType = getHandleType(undefined, handleToCheck);
        const handleNodeId = handleToCheck.getAttribute('data-nodeid');
        const handleId = handleToCheck.getAttribute('data-handleid');
        const connectable = handleToCheck.classList.contains('connectable');
        const connectableEnd = handleToCheck.classList.contains('connectableend');
        if (!handleNodeId || !handleType) {
            return result;
        }
        const connection = {
            source: isTarget ? handleNodeId : fromNodeId,
            sourceHandle: isTarget ? handleId : fromHandleId,
            target: isTarget ? fromNodeId : handleNodeId,
            targetHandle: isTarget ? fromHandleId : handleId,
        };
        result.connection = connection;
        const isConnectable = connectable && connectableEnd;
        // in strict mode we don't allow target to target or source to source connections
        const isValid = isConnectable &&
            (connectionMode === ConnectionMode.Strict
                ? (isTarget && handleType === 'source') || (!isTarget && handleType === 'target')
                : handleNodeId !== fromNodeId || handleId !== fromHandleId);
        result.isValid = isValid && isValidConnection(connection);
        result.toHandle = getHandle(handleNodeId, handleType, handleId, nodeLookup, connectionMode, true);
    }
    return result;
}
const XYHandle = {
    onPointerDown,
    isValid: isValidHandle,
};

function XYMinimap({ domNode, panZoom, getTransform, getViewScale }) {
    const selection = select(domNode);
    function update({ translateExtent, width, height, zoomStep = 10, pannable = true, zoomable = true, inversePan = false, }) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const zoomHandler = (event) => {
            const transform = getTransform();
            if (event.sourceEvent.type !== 'wheel' || !panZoom) {
                return;
            }
            const pinchDelta = -event.sourceEvent.deltaY *
                (event.sourceEvent.deltaMode === 1 ? 0.05 : event.sourceEvent.deltaMode ? 1 : 0.002) *
                zoomStep;
            const nextZoom = transform[2] * Math.pow(2, pinchDelta);
            panZoom.scaleTo(nextZoom);
        };
        let panStart = [0, 0];
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const panStartHandler = (event) => {
            if (event.sourceEvent.type === 'mousedown' || event.sourceEvent.type === 'touchstart') {
                panStart = [
                    event.sourceEvent.clientX ?? event.sourceEvent.touches[0].clientX,
                    event.sourceEvent.clientY ?? event.sourceEvent.touches[0].clientY,
                ];
            }
        };
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const panHandler = (event) => {
            const transform = getTransform();
            if ((event.sourceEvent.type !== 'mousemove' && event.sourceEvent.type !== 'touchmove') || !panZoom) {
                return;
            }
            const panCurrent = [
                event.sourceEvent.clientX ?? event.sourceEvent.touches[0].clientX,
                event.sourceEvent.clientY ?? event.sourceEvent.touches[0].clientY,
            ];
            const panDelta = [panCurrent[0] - panStart[0], panCurrent[1] - panStart[1]];
            panStart = panCurrent;
            const moveScale = getViewScale() * Math.max(transform[2], Math.log(transform[2])) * (inversePan ? -1 : 1);
            const position = {
                x: transform[0] - panDelta[0] * moveScale,
                y: transform[1] - panDelta[1] * moveScale,
            };
            const extent = [
                [0, 0],
                [width, height],
            ];
            panZoom.setViewportConstrained({
                x: position.x,
                y: position.y,
                zoom: transform[2],
            }, extent, translateExtent);
        };
        const zoomAndPanHandler = zoom()
            .on('start', panStartHandler)
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            .on('zoom', pannable ? panHandler : null)
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            .on('zoom.wheel', zoomable ? zoomHandler : null);
        selection.call(zoomAndPanHandler, {});
    }
    function destroy() {
        selection.on('zoom', null);
    }
    return {
        update,
        destroy,
        pointer,
    };
}

/* eslint-disable @typescript-eslint/no-explicit-any */
const viewChanged = (prevViewport, eventViewport) => prevViewport.x !== eventViewport.x || prevViewport.y !== eventViewport.y || prevViewport.zoom !== eventViewport.k;
const transformToViewport = (transform) => ({
    x: transform.x,
    y: transform.y,
    zoom: transform.k,
});
const viewportToTransform = ({ x, y, zoom }) => zoomIdentity.translate(x, y).scale(zoom);
const isWrappedWithClass = (event, className) => event.target.closest(`.${className}`);
const isRightClickPan = (panOnDrag, usedButton) => usedButton === 2 && Array.isArray(panOnDrag) && panOnDrag.includes(2);
// taken from d3-ease: https://github.com/d3/d3-ease/blob/main/src/cubic.js
const defaultEase = (t) => ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2;
const getD3Transition = (selection, duration = 0, ease = defaultEase, onEnd = () => { }) => {
    const hasDuration = typeof duration === 'number' && duration > 0;
    if (!hasDuration) {
        onEnd();
    }
    return hasDuration ? selection.transition().duration(duration).ease(ease).on('end', onEnd) : selection;
};
const wheelDelta = (event) => {
    const factor = event.ctrlKey && isMacOs() ? 10 : 1;
    return -event.deltaY * (event.deltaMode === 1 ? 0.05 : event.deltaMode ? 1 : 0.002) * factor;
};

function createPanOnScrollHandler({ zoomPanValues, noWheelClassName, d3Selection, d3Zoom, panOnScrollMode, panOnScrollSpeed, zoomOnPinch, onPanZoomStart, onPanZoom, onPanZoomEnd, }) {
    return (event) => {
        if (isWrappedWithClass(event, noWheelClassName)) {
            return false;
        }
        event.preventDefault();
        event.stopImmediatePropagation();
        const currentZoom = d3Selection.property('__zoom').k || 1;
        // macos sets ctrlKey=true for pinch gesture on a trackpad
        if (event.ctrlKey && zoomOnPinch) {
            const point = pointer(event);
            const pinchDelta = wheelDelta(event);
            const zoom = currentZoom * Math.pow(2, pinchDelta);
            // @ts-ignore
            d3Zoom.scaleTo(d3Selection, zoom, point, event);
            return;
        }
        /*
         * increase scroll speed in firefox
         * firefox: deltaMode === 1; chrome: deltaMode === 0
         */
        const deltaNormalize = event.deltaMode === 1 ? 20 : 1;
        let deltaX = panOnScrollMode === PanOnScrollMode.Vertical ? 0 : event.deltaX * deltaNormalize;
        let deltaY = panOnScrollMode === PanOnScrollMode.Horizontal ? 0 : event.deltaY * deltaNormalize;
        // this enables vertical scrolling with shift + scroll on windows
        if (!isMacOs() && event.shiftKey && panOnScrollMode !== PanOnScrollMode.Vertical) {
            deltaX = event.deltaY * deltaNormalize;
            deltaY = 0;
        }
        d3Zoom.translateBy(d3Selection, -(deltaX / currentZoom) * panOnScrollSpeed, -(deltaY / currentZoom) * panOnScrollSpeed, 
        // @ts-ignore
        { internal: true });
        const nextViewport = transformToViewport(d3Selection.property('__zoom'));
        clearTimeout(zoomPanValues.panScrollTimeout);
        /*
         * for pan on scroll we need to handle the event calls on our own
         * we can't use the start, zoom and end events from d3-zoom
         * because start and move gets called on every scroll event and not once at the beginning
         */
        if (!zoomPanValues.isPanScrolling) {
            zoomPanValues.isPanScrolling = true;
            onPanZoomStart?.(event, nextViewport);
        }
        if (zoomPanValues.isPanScrolling) {
            onPanZoom?.(event, nextViewport);
            zoomPanValues.panScrollTimeout = setTimeout(() => {
                onPanZoomEnd?.(event, nextViewport);
                zoomPanValues.isPanScrolling = false;
            }, 150);
        }
    };
}
function createZoomOnScrollHandler({ noWheelClassName, preventScrolling, d3ZoomHandler }) {
    return function (event, d) {
        const isWheel = event.type === 'wheel';
        // we still want to enable pinch zooming even if preventScrolling is set to false
        const preventZoom = !preventScrolling && isWheel && !event.ctrlKey;
        const hasNoWheelClass = isWrappedWithClass(event, noWheelClassName);
        // if user is pinch zooming above a nowheel element, we don't want the browser to zoom
        if (event.ctrlKey && isWheel && hasNoWheelClass) {
            event.preventDefault();
        }
        if (preventZoom || hasNoWheelClass) {
            return null;
        }
        event.preventDefault();
        d3ZoomHandler.call(this, event, d);
    };
}
function createPanZoomStartHandler({ zoomPanValues, onDraggingChange, onPanZoomStart }) {
    return (event) => {
        if (event.sourceEvent?.internal) {
            return;
        }
        const viewport = transformToViewport(event.transform);
        // we need to remember it here, because it's always 0 in the "zoom" event
        zoomPanValues.mouseButton = event.sourceEvent?.button || 0;
        zoomPanValues.isZoomingOrPanning = true;
        zoomPanValues.prevViewport = viewport;
        if (event.sourceEvent?.type === 'mousedown') {
            onDraggingChange(true);
        }
        if (onPanZoomStart) {
            onPanZoomStart?.(event.sourceEvent, viewport);
        }
    };
}
function createPanZoomHandler({ zoomPanValues, panOnDrag, onPaneContextMenu, onTransformChange, onPanZoom, }) {
    return (event) => {
        zoomPanValues.usedRightMouseButton = !!(onPaneContextMenu && isRightClickPan(panOnDrag, zoomPanValues.mouseButton ?? 0));
        if (!event.sourceEvent?.sync) {
            onTransformChange([event.transform.x, event.transform.y, event.transform.k]);
        }
        if (onPanZoom && !event.sourceEvent?.internal) {
            onPanZoom?.(event.sourceEvent, transformToViewport(event.transform));
        }
    };
}
function createPanZoomEndHandler({ zoomPanValues, panOnDrag, panOnScroll, onDraggingChange, onPanZoomEnd, onPaneContextMenu, }) {
    return (event) => {
        if (event.sourceEvent?.internal) {
            return;
        }
        zoomPanValues.isZoomingOrPanning = false;
        if (onPaneContextMenu &&
            isRightClickPan(panOnDrag, zoomPanValues.mouseButton ?? 0) &&
            !zoomPanValues.usedRightMouseButton &&
            event.sourceEvent) {
            onPaneContextMenu(event.sourceEvent);
        }
        zoomPanValues.usedRightMouseButton = false;
        onDraggingChange(false);
        if (onPanZoomEnd && viewChanged(zoomPanValues.prevViewport, event.transform)) {
            const viewport = transformToViewport(event.transform);
            zoomPanValues.prevViewport = viewport;
            clearTimeout(zoomPanValues.timerId);
            zoomPanValues.timerId = setTimeout(() => {
                onPanZoomEnd?.(event.sourceEvent, viewport);
            }, 
            // we need a setTimeout for panOnScroll to supress multiple end events fired during scroll
            panOnScroll ? 150 : 0);
        }
    };
}

/* eslint-disable @typescript-eslint/no-explicit-any */
function createFilter({ zoomActivationKeyPressed, zoomOnScroll, zoomOnPinch, panOnDrag, panOnScroll, zoomOnDoubleClick, userSelectionActive, noWheelClassName, noPanClassName, lib, }) {
    return (event) => {
        const zoomScroll = zoomActivationKeyPressed || zoomOnScroll;
        const pinchZoom = zoomOnPinch && event.ctrlKey;
        if (event.button === 1 &&
            event.type === 'mousedown' &&
            (isWrappedWithClass(event, `${lib}-flow__node`) || isWrappedWithClass(event, `${lib}-flow__edge`))) {
            return true;
        }
        // if all interactions are disabled, we prevent all zoom events
        if (!panOnDrag && !zoomScroll && !panOnScroll && !zoomOnDoubleClick && !zoomOnPinch) {
            return false;
        }
        // during a selection we prevent all other interactions
        if (userSelectionActive) {
            return false;
        }
        // if the target element is inside an element with the nowheel class, we prevent zooming
        if (isWrappedWithClass(event, noWheelClassName) && event.type === 'wheel') {
            return false;
        }
        // if the target element is inside an element with the nopan class, we prevent panning
        if (isWrappedWithClass(event, noPanClassName) &&
            (event.type !== 'wheel' || (panOnScroll && event.type === 'wheel' && !zoomActivationKeyPressed))) {
            return false;
        }
        if (!zoomOnPinch && event.ctrlKey && event.type === 'wheel') {
            return false;
        }
        if (!zoomOnPinch && event.type === 'touchstart' && event.touches?.length > 1) {
            event.preventDefault(); // if you manage to start with 2 touches, we prevent native zoom
            return false;
        }
        // when there is no scroll handling enabled, we prevent all wheel events
        if (!zoomScroll && !panOnScroll && !pinchZoom && event.type === 'wheel') {
            return false;
        }
        // if the pane is not movable, we prevent dragging it with mousestart or touchstart
        if (!panOnDrag && (event.type === 'mousedown' || event.type === 'touchstart')) {
            return false;
        }
        // if the pane is only movable using allowed clicks
        if (Array.isArray(panOnDrag) && !panOnDrag.includes(event.button) && event.type === 'mousedown') {
            return false;
        }
        // We only allow right clicks if pan on drag is set to right click
        const buttonAllowed = (Array.isArray(panOnDrag) && panOnDrag.includes(event.button)) || !event.button || event.button <= 1;
        // default filter for d3-zoom
        return (!event.ctrlKey || event.type === 'wheel') && buttonAllowed;
    };
}

function XYPanZoom({ domNode, minZoom, maxZoom, paneClickDistance, translateExtent, viewport, onPanZoom, onPanZoomStart, onPanZoomEnd, onDraggingChange, }) {
    const zoomPanValues = {
        isZoomingOrPanning: false,
        usedRightMouseButton: false,
        prevViewport: { x: 0, y: 0, zoom: 0 },
        mouseButton: 0,
        timerId: undefined,
        panScrollTimeout: undefined,
        isPanScrolling: false,
    };
    const bbox = domNode.getBoundingClientRect();
    const d3ZoomInstance = zoom()
        .clickDistance(!isNumeric(paneClickDistance) || paneClickDistance < 0 ? 0 : paneClickDistance)
        .scaleExtent([minZoom, maxZoom])
        .translateExtent(translateExtent);
    const d3Selection = select(domNode).call(d3ZoomInstance);
    setViewportConstrained({
        x: viewport.x,
        y: viewport.y,
        zoom: clamp(viewport.zoom, minZoom, maxZoom),
    }, [
        [0, 0],
        [bbox.width, bbox.height],
    ], translateExtent);
    const d3ZoomHandler = d3Selection.on('wheel.zoom');
    const d3DblClickZoomHandler = d3Selection.on('dblclick.zoom');
    d3ZoomInstance.wheelDelta(wheelDelta);
    function setTransform(transform, options) {
        if (d3Selection) {
            return new Promise((resolve) => {
                d3ZoomInstance?.interpolate(options?.interpolate === 'linear' ? interpolate : interpolateZoom).transform(getD3Transition(d3Selection, options?.duration, options?.ease, () => resolve(true)), transform);
            });
        }
        return Promise.resolve(false);
    }
    // public functions
    function update({ noWheelClassName, noPanClassName, onPaneContextMenu, userSelectionActive, panOnScroll, panOnDrag, panOnScrollMode, panOnScrollSpeed, preventScrolling, zoomOnPinch, zoomOnScroll, zoomOnDoubleClick, zoomActivationKeyPressed, lib, onTransformChange, }) {
        if (userSelectionActive && !zoomPanValues.isZoomingOrPanning) {
            destroy();
        }
        const isPanOnScroll = panOnScroll && !zoomActivationKeyPressed && !userSelectionActive;
        const wheelHandler = isPanOnScroll
            ? createPanOnScrollHandler({
                zoomPanValues,
                noWheelClassName,
                d3Selection,
                d3Zoom: d3ZoomInstance,
                panOnScrollMode,
                panOnScrollSpeed,
                zoomOnPinch,
                onPanZoomStart,
                onPanZoom,
                onPanZoomEnd,
            })
            : createZoomOnScrollHandler({
                noWheelClassName,
                preventScrolling,
                d3ZoomHandler,
            });
        d3Selection.on('wheel.zoom', wheelHandler, { passive: false });
        if (!userSelectionActive) {
            // pan zoom start
            const startHandler = createPanZoomStartHandler({
                zoomPanValues,
                onDraggingChange,
                onPanZoomStart,
            });
            d3ZoomInstance.on('start', startHandler);
            // pan zoom
            const panZoomHandler = createPanZoomHandler({
                zoomPanValues,
                panOnDrag,
                onPaneContextMenu: !!onPaneContextMenu,
                onPanZoom,
                onTransformChange,
            });
            d3ZoomInstance.on('zoom', panZoomHandler);
            // pan zoom end
            const panZoomEndHandler = createPanZoomEndHandler({
                zoomPanValues,
                panOnDrag,
                panOnScroll,
                onPaneContextMenu,
                onPanZoomEnd,
                onDraggingChange,
            });
            d3ZoomInstance.on('end', panZoomEndHandler);
        }
        const filter = createFilter({
            zoomActivationKeyPressed,
            panOnDrag,
            zoomOnScroll,
            panOnScroll,
            zoomOnDoubleClick,
            zoomOnPinch,
            userSelectionActive,
            noPanClassName,
            noWheelClassName,
            lib,
        });
        d3ZoomInstance.filter(filter);
        /*
         * We cannot add zoomOnDoubleClick to the filter above because
         * double tapping on touch screens circumvents the filter and
         * dblclick.zoom is fired on the selection directly
         */
        if (zoomOnDoubleClick) {
            d3Selection.on('dblclick.zoom', d3DblClickZoomHandler);
        }
        else {
            d3Selection.on('dblclick.zoom', null);
        }
    }
    function destroy() {
        d3ZoomInstance.on('zoom', null);
    }
    async function setViewportConstrained(viewport, extent, translateExtent) {
        const nextTransform = viewportToTransform(viewport);
        const contrainedTransform = d3ZoomInstance?.constrain()(nextTransform, extent, translateExtent);
        if (contrainedTransform) {
            await setTransform(contrainedTransform);
        }
        return new Promise((resolve) => resolve(contrainedTransform));
    }
    async function setViewport(viewport, options) {
        const nextTransform = viewportToTransform(viewport);
        await setTransform(nextTransform, options);
        return new Promise((resolve) => resolve(nextTransform));
    }
    function syncViewport(viewport) {
        if (d3Selection) {
            const nextTransform = viewportToTransform(viewport);
            const currentTransform = d3Selection.property('__zoom');
            if (currentTransform.k !== viewport.zoom ||
                currentTransform.x !== viewport.x ||
                currentTransform.y !== viewport.y) {
                // eslint-disable-next-line @typescript-eslint/ban-ts-comment
                // @ts-ignore
                d3ZoomInstance?.transform(d3Selection, nextTransform, null, { sync: true });
            }
        }
    }
    function getViewport() {
        const transform = d3Selection ? zoomTransform(d3Selection.node()) : { x: 0, y: 0, k: 1 };
        return { x: transform.x, y: transform.y, zoom: transform.k };
    }
    function scaleTo(zoom, options) {
        if (d3Selection) {
            return new Promise((resolve) => {
                d3ZoomInstance?.interpolate(options?.interpolate === 'linear' ? interpolate : interpolateZoom).scaleTo(getD3Transition(d3Selection, options?.duration, options?.ease, () => resolve(true)), zoom);
            });
        }
        return Promise.resolve(false);
    }
    function scaleBy(factor, options) {
        if (d3Selection) {
            return new Promise((resolve) => {
                d3ZoomInstance?.interpolate(options?.interpolate === 'linear' ? interpolate : interpolateZoom).scaleBy(getD3Transition(d3Selection, options?.duration, options?.ease, () => resolve(true)), factor);
            });
        }
        return Promise.resolve(false);
    }
    function setScaleExtent(scaleExtent) {
        d3ZoomInstance?.scaleExtent(scaleExtent);
    }
    function setTranslateExtent(translateExtent) {
        d3ZoomInstance?.translateExtent(translateExtent);
    }
    function setClickDistance(distance) {
        const validDistance = !isNumeric(distance) || distance < 0 ? 0 : distance;
        d3ZoomInstance?.clickDistance(validDistance);
    }
    return {
        update,
        destroy,
        setViewport,
        setViewportConstrained,
        getViewport,
        scaleTo,
        scaleBy,
        setScaleExtent,
        setTranslateExtent,
        syncViewport,
        setClickDistance,
    };
}

/**
 * Used to determine the variant of the resize control
 *
 * @public
 */
var ResizeControlVariant;
(function (ResizeControlVariant) {
    ResizeControlVariant["Line"] = "line";
    ResizeControlVariant["Handle"] = "handle";
})(ResizeControlVariant || (ResizeControlVariant = {}));

/**
 * Get all connecting edges for a given set of nodes
 * @param width - new width of the node
 * @param prevWidth - previous width of the node
 * @param height - new height of the node
 * @param prevHeight - previous height of the node
 * @param affectsX - whether to invert the resize direction for the x axis
 * @param affectsY - whether to invert the resize direction for the y axis
 * @returns array of two numbers representing the direction of the resize for each axis, 0 = no change, 1 = increase, -1 = decrease
 */
function getResizeDirection({ width, prevWidth, height, prevHeight, affectsX, affectsY, }) {
    const deltaWidth = width - prevWidth;
    const deltaHeight = height - prevHeight;
    const direction = [deltaWidth > 0 ? 1 : deltaWidth < 0 ? -1 : 0, deltaHeight > 0 ? 1 : deltaHeight < 0 ? -1 : 0];
    if (deltaWidth && affectsX) {
        direction[0] = direction[0] * -1;
    }
    if (deltaHeight && affectsY) {
        direction[1] = direction[1] * -1;
    }
    return direction;
}
/**
 * Parses the control position that is being dragged to dimensions that are being resized
 * @param controlPosition - position of the control that is being dragged
 * @returns isHorizontal, isVertical, affectsX, affectsY,
 */
function getControlDirection(controlPosition) {
    const isHorizontal = controlPosition.includes('right') || controlPosition.includes('left');
    const isVertical = controlPosition.includes('bottom') || controlPosition.includes('top');
    const affectsX = controlPosition.includes('left');
    const affectsY = controlPosition.includes('top');
    return {
        isHorizontal,
        isVertical,
        affectsX,
        affectsY,
    };
}
function getLowerExtentClamp(lowerExtent, lowerBound) {
    return Math.max(0, lowerBound - lowerExtent);
}
function getUpperExtentClamp(upperExtent, upperBound) {
    return Math.max(0, upperExtent - upperBound);
}
function getSizeClamp(size, minSize, maxSize) {
    return Math.max(0, minSize - size, size - maxSize);
}
function xor(a, b) {
    return a ? !b : b;
}
/**
 * Calculates new width & height and x & y of node after resize based on pointer position
 * @description - Buckle up, this is a chunky one... If you want to determine the new dimensions of a node after a resize,
 * you have to account for all possible restrictions: min/max width/height of the node, the maximum extent the node is allowed
 * to move in (in this case: resize into) determined by the parent node, the minimal extent determined by child nodes
 * with expandParent or extent: 'parent' set and oh yeah, these things also have to work with keepAspectRatio!
 * The way this is done is by determining how much each of these restricting actually restricts the resize and then applying the
 * strongest restriction. Because the resize affects x, y and width, height and width, height of a opposing side with keepAspectRatio,
 * the resize amount is always kept in distX & distY amount (the distance in mouse movement)
 * Instead of clamping each value, we first calculate the biggest 'clamp' (for the lack of a better name) and then apply it to all values.
 * To complicate things nodeOrigin has to be taken into account as well. This is done by offsetting the nodes as if their origin is [0, 0],
 * then calculating the restrictions as usual
 * @param startValues - starting values of resize
 * @param controlDirection - dimensions affected by the resize
 * @param pointerPosition - the current pointer position corrected for snapping
 * @param boundaries - minimum and maximum dimensions of the node
 * @param keepAspectRatio - prevent changes of asprect ratio
 * @returns x, y, width and height of the node after resize
 */
function getDimensionsAfterResize(startValues, controlDirection, pointerPosition, boundaries, keepAspectRatio, nodeOrigin, extent, childExtent) {
    let { affectsX, affectsY } = controlDirection;
    const { isHorizontal, isVertical } = controlDirection;
    const isDiagonal = isHorizontal && isVertical;
    const { xSnapped, ySnapped } = pointerPosition;
    const { minWidth, maxWidth, minHeight, maxHeight } = boundaries;
    const { x: startX, y: startY, width: startWidth, height: startHeight, aspectRatio } = startValues;
    let distX = Math.floor(isHorizontal ? xSnapped - startValues.pointerX : 0);
    let distY = Math.floor(isVertical ? ySnapped - startValues.pointerY : 0);
    const newWidth = startWidth + (affectsX ? -distX : distX);
    const newHeight = startHeight + (affectsY ? -distY : distY);
    const originOffsetX = -nodeOrigin[0] * startWidth;
    const originOffsetY = -nodeOrigin[1] * startHeight;
    // Check if maxWidth, minWWidth, maxHeight, minHeight are restricting the resize
    let clampX = getSizeClamp(newWidth, minWidth, maxWidth);
    let clampY = getSizeClamp(newHeight, minHeight, maxHeight);
    // Check if extent is restricting the resize
    if (extent) {
        let xExtentClamp = 0;
        let yExtentClamp = 0;
        if (affectsX && distX < 0) {
            xExtentClamp = getLowerExtentClamp(startX + distX + originOffsetX, extent[0][0]);
        }
        else if (!affectsX && distX > 0) {
            xExtentClamp = getUpperExtentClamp(startX + newWidth + originOffsetX, extent[1][0]);
        }
        if (affectsY && distY < 0) {
            yExtentClamp = getLowerExtentClamp(startY + distY + originOffsetY, extent[0][1]);
        }
        else if (!affectsY && distY > 0) {
            yExtentClamp = getUpperExtentClamp(startY + newHeight + originOffsetY, extent[1][1]);
        }
        clampX = Math.max(clampX, xExtentClamp);
        clampY = Math.max(clampY, yExtentClamp);
    }
    // Check if the child extent is restricting the resize
    if (childExtent) {
        let xExtentClamp = 0;
        let yExtentClamp = 0;
        if (affectsX && distX > 0) {
            xExtentClamp = getUpperExtentClamp(startX + distX, childExtent[0][0]);
        }
        else if (!affectsX && distX < 0) {
            xExtentClamp = getLowerExtentClamp(startX + newWidth, childExtent[1][0]);
        }
        if (affectsY && distY > 0) {
            yExtentClamp = getUpperExtentClamp(startY + distY, childExtent[0][1]);
        }
        else if (!affectsY && distY < 0) {
            yExtentClamp = getLowerExtentClamp(startY + newHeight, childExtent[1][1]);
        }
        clampX = Math.max(clampX, xExtentClamp);
        clampY = Math.max(clampY, yExtentClamp);
    }
    // Check if the aspect ratio resizing of the other side is restricting the resize
    if (keepAspectRatio) {
        if (isHorizontal) {
            // Check if the max dimensions might be restricting the resize
            const aspectHeightClamp = getSizeClamp(newWidth / aspectRatio, minHeight, maxHeight) * aspectRatio;
            clampX = Math.max(clampX, aspectHeightClamp);
            // Check if the extent is restricting the resize
            if (extent) {
                let aspectExtentClamp = 0;
                if ((!affectsX && !affectsY) || (affectsX && !affectsY && isDiagonal)) {
                    aspectExtentClamp =
                        getUpperExtentClamp(startY + originOffsetY + newWidth / aspectRatio, extent[1][1]) * aspectRatio;
                }
                else {
                    aspectExtentClamp =
                        getLowerExtentClamp(startY + originOffsetY + (affectsX ? distX : -distX) / aspectRatio, extent[0][1]) *
                            aspectRatio;
                }
                clampX = Math.max(clampX, aspectExtentClamp);
            }
            // Check if the child extent is restricting the resize
            if (childExtent) {
                let aspectExtentClamp = 0;
                if ((!affectsX && !affectsY) || (affectsX && !affectsY && isDiagonal)) {
                    aspectExtentClamp = getLowerExtentClamp(startY + newWidth / aspectRatio, childExtent[1][1]) * aspectRatio;
                }
                else {
                    aspectExtentClamp =
                        getUpperExtentClamp(startY + (affectsX ? distX : -distX) / aspectRatio, childExtent[0][1]) * aspectRatio;
                }
                clampX = Math.max(clampX, aspectExtentClamp);
            }
        }
        // Do the same thing for vertical resizing
        if (isVertical) {
            const aspectWidthClamp = getSizeClamp(newHeight * aspectRatio, minWidth, maxWidth) / aspectRatio;
            clampY = Math.max(clampY, aspectWidthClamp);
            if (extent) {
                let aspectExtentClamp = 0;
                if ((!affectsX && !affectsY) || (affectsY && !affectsX && isDiagonal)) {
                    aspectExtentClamp =
                        getUpperExtentClamp(startX + newHeight * aspectRatio + originOffsetX, extent[1][0]) / aspectRatio;
                }
                else {
                    aspectExtentClamp =
                        getLowerExtentClamp(startX + (affectsY ? distY : -distY) * aspectRatio + originOffsetX, extent[0][0]) /
                            aspectRatio;
                }
                clampY = Math.max(clampY, aspectExtentClamp);
            }
            if (childExtent) {
                let aspectExtentClamp = 0;
                if ((!affectsX && !affectsY) || (affectsY && !affectsX && isDiagonal)) {
                    aspectExtentClamp = getLowerExtentClamp(startX + newHeight * aspectRatio, childExtent[1][0]) / aspectRatio;
                }
                else {
                    aspectExtentClamp =
                        getUpperExtentClamp(startX + (affectsY ? distY : -distY) * aspectRatio, childExtent[0][0]) / aspectRatio;
                }
                clampY = Math.max(clampY, aspectExtentClamp);
            }
        }
    }
    distY = distY + (distY < 0 ? clampY : -clampY);
    distX = distX + (distX < 0 ? clampX : -clampX);
    if (keepAspectRatio) {
        if (isDiagonal) {
            if (newWidth > newHeight * aspectRatio) {
                distY = (xor(affectsX, affectsY) ? -distX : distX) / aspectRatio;
            }
            else {
                distX = (xor(affectsX, affectsY) ? -distY : distY) * aspectRatio;
            }
        }
        else {
            if (isHorizontal) {
                distY = distX / aspectRatio;
                affectsY = affectsX;
            }
            else {
                distX = distY * aspectRatio;
                affectsX = affectsY;
            }
        }
    }
    const x = affectsX ? startX + distX : startX;
    const y = affectsY ? startY + distY : startY;
    return {
        width: startWidth + (affectsX ? -distX : distX),
        height: startHeight + (affectsY ? -distY : distY),
        x: nodeOrigin[0] * distX * (!affectsX ? 1 : -1) + x,
        y: nodeOrigin[1] * distY * (!affectsY ? 1 : -1) + y,
    };
}

const initPrevValues$1 = { width: 0, height: 0, x: 0, y: 0 };
const initStartValues = {
    ...initPrevValues$1,
    pointerX: 0,
    pointerY: 0,
    aspectRatio: 1,
};
function nodeToParentExtent(node) {
    return [
        [0, 0],
        [node.measured.width, node.measured.height],
    ];
}
function nodeToChildExtent(child, parent, nodeOrigin) {
    const x = parent.position.x + child.position.x;
    const y = parent.position.y + child.position.y;
    const width = child.measured.width ?? 0;
    const height = child.measured.height ?? 0;
    const originOffsetX = nodeOrigin[0] * width;
    const originOffsetY = nodeOrigin[1] * height;
    return [
        [x - originOffsetX, y - originOffsetY],
        [x + width - originOffsetX, y + height - originOffsetY],
    ];
}
function XYResizer({ domNode, nodeId, getStoreItems, onChange, onEnd }) {
    const selection = select(domNode);
    function update({ controlPosition, boundaries, keepAspectRatio, resizeDirection, onResizeStart, onResize, onResizeEnd, shouldResize, }) {
        let prevValues = { ...initPrevValues$1 };
        let startValues = { ...initStartValues };
        const controlDirection = getControlDirection(controlPosition);
        let node = undefined;
        let containerBounds = null;
        let childNodes = [];
        let parentNode = undefined; // Needed to fix expandParent
        let parentExtent = undefined;
        let childExtent = undefined;
        const dragHandler = drag()
            .on('start', (event) => {
            const { nodeLookup, transform, snapGrid, snapToGrid, nodeOrigin, paneDomNode } = getStoreItems();
            node = nodeLookup.get(nodeId);
            if (!node) {
                return;
            }
            containerBounds = paneDomNode?.getBoundingClientRect() ?? null;
            const { xSnapped, ySnapped } = getPointerPosition(event.sourceEvent, {
                transform,
                snapGrid,
                snapToGrid,
                containerBounds,
            });
            prevValues = {
                width: node.measured.width ?? 0,
                height: node.measured.height ?? 0,
                x: node.position.x ?? 0,
                y: node.position.y ?? 0,
            };
            startValues = {
                ...prevValues,
                pointerX: xSnapped,
                pointerY: ySnapped,
                aspectRatio: prevValues.width / prevValues.height,
            };
            parentNode = undefined;
            if (node.parentId && (node.extent === 'parent' || node.expandParent)) {
                parentNode = nodeLookup.get(node.parentId);
                parentExtent = parentNode && node.extent === 'parent' ? nodeToParentExtent(parentNode) : undefined;
            }
            /*
             * Collect all child nodes to correct their relative positions when top/left changes
             * Determine largest minimal extent the parent node is allowed to resize to
             */
            childNodes = [];
            childExtent = undefined;
            for (const [childId, child] of nodeLookup) {
                if (child.parentId === nodeId) {
                    childNodes.push({
                        id: childId,
                        position: { ...child.position },
                        extent: child.extent,
                    });
                    if (child.extent === 'parent' || child.expandParent) {
                        const extent = nodeToChildExtent(child, node, child.origin ?? nodeOrigin);
                        if (childExtent) {
                            childExtent = [
                                [Math.min(extent[0][0], childExtent[0][0]), Math.min(extent[0][1], childExtent[0][1])],
                                [Math.max(extent[1][0], childExtent[1][0]), Math.max(extent[1][1], childExtent[1][1])],
                            ];
                        }
                        else {
                            childExtent = extent;
                        }
                    }
                }
            }
            onResizeStart?.(event, { ...prevValues });
        })
            .on('drag', (event) => {
            const { transform, snapGrid, snapToGrid, nodeOrigin: storeNodeOrigin } = getStoreItems();
            const pointerPosition = getPointerPosition(event.sourceEvent, {
                transform,
                snapGrid,
                snapToGrid,
                containerBounds,
            });
            const childChanges = [];
            if (!node) {
                return;
            }
            const { x: prevX, y: prevY, width: prevWidth, height: prevHeight } = prevValues;
            const change = {};
            const nodeOrigin = node.origin ?? storeNodeOrigin;
            const { width, height, x, y } = getDimensionsAfterResize(startValues, controlDirection, pointerPosition, boundaries, keepAspectRatio, nodeOrigin, parentExtent, childExtent);
            const isWidthChange = width !== prevWidth;
            const isHeightChange = height !== prevHeight;
            const isXPosChange = x !== prevX && isWidthChange;
            const isYPosChange = y !== prevY && isHeightChange;
            if (!isXPosChange && !isYPosChange && !isWidthChange && !isHeightChange) {
                return;
            }
            if (isXPosChange || isYPosChange || nodeOrigin[0] === 1 || nodeOrigin[1] === 1) {
                change.x = isXPosChange ? x : prevValues.x;
                change.y = isYPosChange ? y : prevValues.y;
                prevValues.x = change.x;
                prevValues.y = change.y;
                /*
                 * when top/left changes, correct the relative positions of child nodes
                 * so that they stay in the same position
                 */
                if (childNodes.length > 0) {
                    const xChange = x - prevX;
                    const yChange = y - prevY;
                    for (const childNode of childNodes) {
                        childNode.position = {
                            x: childNode.position.x - xChange + nodeOrigin[0] * (width - prevWidth),
                            y: childNode.position.y - yChange + nodeOrigin[1] * (height - prevHeight),
                        };
                        childChanges.push(childNode);
                    }
                }
            }
            if (isWidthChange || isHeightChange) {
                change.width =
                    isWidthChange && (!resizeDirection || resizeDirection === 'horizontal') ? width : prevValues.width;
                change.height =
                    isHeightChange && (!resizeDirection || resizeDirection === 'vertical') ? height : prevValues.height;
                prevValues.width = change.width;
                prevValues.height = change.height;
            }
            // Fix expandParent when resizing from top/left
            if (parentNode && node.expandParent) {
                const xLimit = nodeOrigin[0] * (change.width ?? 0);
                if (change.x && change.x < xLimit) {
                    prevValues.x = xLimit;
                    startValues.x = startValues.x - (change.x - xLimit);
                }
                const yLimit = nodeOrigin[1] * (change.height ?? 0);
                if (change.y && change.y < yLimit) {
                    prevValues.y = yLimit;
                    startValues.y = startValues.y - (change.y - yLimit);
                }
            }
            const direction = getResizeDirection({
                width: prevValues.width,
                prevWidth,
                height: prevValues.height,
                prevHeight,
                affectsX: controlDirection.affectsX,
                affectsY: controlDirection.affectsY,
            });
            const nextValues = { ...prevValues, direction };
            const callResize = shouldResize?.(event, nextValues);
            if (callResize === false) {
                return;
            }
            onResize?.(event, nextValues);
            onChange(change, childChanges);
        })
            .on('end', (event) => {
            onResizeEnd?.(event, { ...prevValues });
            onEnd?.({ ...prevValues });
        });
        selection.call(dragHandler);
    }
    function destroy() {
        selection.on('.drag', null);
    }
    return {
        update,
        destroy,
    };
}

const StoreContext = createContext(null);
const Provider$1 = StoreContext.Provider;

const zustandErrorMessage = errorMessages['error001']();
/**
 * This hook can be used to subscribe to internal state changes of the React Flow
 * component. The `useStore` hook is re-exported from the [Zustand](https://github.com/pmndrs/zustand)
 * state management library, so you should check out their docs for more details.
 *
 * @public
 * @param selector - A selector function that returns a slice of the flow's internal state.
 * Extracting or transforming just the state you need is a good practice to avoid unnecessary
 * re-renders.
 * @param equalityFn - A function to compare the previous and next value. This is incredibly useful
 * for preventing unnecessary re-renders. Good sensible defaults are using `Object.is` or importing
 * `zustand/shallow`, but you can be as granular as you like.
 * @returns The selected state slice.
 *
 * @example
 * ```ts
 * const nodes = useStore((state) => state.nodes);
 * ```
 *
 * @remarks This hook should only be used if there is no other way to access the internal
 * state. For many of the common use cases, there are dedicated hooks available
 * such as {@link useReactFlow}, {@link useViewport}, etc.
 */
function useStore(selector, equalityFn) {
    const store = useContext(StoreContext);
    if (store === null) {
        throw new Error(zustandErrorMessage);
    }
    return useStoreWithEqualityFn(store, selector, equalityFn);
}
/**
 * In some cases, you might need to access the store directly. This hook returns the store object which can be used on demand to access the state or dispatch actions.
 *
 * @returns The store object.
 * @example
 * ```ts
 * const store = useStoreApi();
 * ```
 *
 * @remarks This hook should only be used if there is no other way to access the internal
 * state. For many of the common use cases, there are dedicated hooks available
 * such as {@link useReactFlow}, {@link useViewport}, etc.
 */
function useStoreApi() {
    const store = useContext(StoreContext);
    if (store === null) {
        throw new Error(zustandErrorMessage);
    }
    return useMemo(() => ({
        getState: store.getState,
        setState: store.setState,
        subscribe: store.subscribe,
    }), [store]);
}

const style = { display: 'none' };
const ariaLiveStyle = {
    position: 'absolute',
    width: 1,
    height: 1,
    margin: -1,
    border: 0,
    padding: 0,
    overflow: 'hidden',
    clip: 'rect(0px, 0px, 0px, 0px)',
    clipPath: 'inset(100%)',
};
const ARIA_NODE_DESC_KEY = 'react-flow__node-desc';
const ARIA_EDGE_DESC_KEY = 'react-flow__edge-desc';
const ARIA_LIVE_MESSAGE = 'react-flow__aria-live';
const ariaLiveSelector = (s) => s.ariaLiveMessage;
const ariaLabelConfigSelector = (s) => s.ariaLabelConfig;
function AriaLiveMessage({ rfId }) {
    const ariaLiveMessage = useStore(ariaLiveSelector);
    return (jsx("div", { id: `${ARIA_LIVE_MESSAGE}-${rfId}`, "aria-live": "assertive", "aria-atomic": "true", style: ariaLiveStyle, children: ariaLiveMessage }));
}
function A11yDescriptions({ rfId, disableKeyboardA11y }) {
    const ariaLabelConfig = useStore(ariaLabelConfigSelector);
    return (jsxs(Fragment, { children: [jsx("div", { id: `${ARIA_NODE_DESC_KEY}-${rfId}`, style: style, children: disableKeyboardA11y
                    ? ariaLabelConfig['node.a11yDescription.default']
                    : ariaLabelConfig['node.a11yDescription.keyboardDisabled'] }), jsx("div", { id: `${ARIA_EDGE_DESC_KEY}-${rfId}`, style: style, children: ariaLabelConfig['edge.a11yDescription.default'] }), !disableKeyboardA11y && jsx(AriaLiveMessage, { rfId: rfId })] }));
}

/**
 * The `<Panel />` component helps you position content above the viewport.
 * It is used internally by the [`<MiniMap />`](/api-reference/components/minimap)
 * and [`<Controls />`](/api-reference/components/controls) components.
 *
 * @public
 *
 * @example
 * ```jsx
 *import { ReactFlow, Background, Panel } from '@xyflow/react';
 *
 *export default function Flow() {
 *  return (
 *    <ReactFlow nodes={[]} fitView>
 *      <Panel position="top-left">top-left</Panel>
 *      <Panel position="top-center">top-center</Panel>
 *      <Panel position="top-right">top-right</Panel>
 *      <Panel position="bottom-left">bottom-left</Panel>
 *      <Panel position="bottom-center">bottom-center</Panel>
 *      <Panel position="bottom-right">bottom-right</Panel>
 *    </ReactFlow>
 *  );
 *}
 *```
 */
const Panel = forwardRef(({ position = 'top-left', children, className, style, ...rest }, ref) => {
    const positionClasses = `${position}`.split('-');
    return (jsx("div", { className: cc(['react-flow__panel', className, ...positionClasses]), style: style, ref: ref, ...rest, children: children }));
});
Panel.displayName = 'Panel';

function Attribution({ proOptions, position = 'bottom-right' }) {
    if (proOptions?.hideAttribution) {
        return null;
    }
    return (jsx(Panel, { position: position, className: "react-flow__attribution", "data-message": "Please only hide this attribution when you are subscribed to React Flow Pro: https://pro.reactflow.dev", children: jsx("a", { href: "https://reactflow.dev", target: "_blank", rel: "noopener noreferrer", "aria-label": "React Flow attribution", children: "React Flow" }) }));
}

const selector$m = (s) => {
    const selectedNodes = [];
    const selectedEdges = [];
    for (const [, node] of s.nodeLookup) {
        if (node.selected) {
            selectedNodes.push(node.internals.userNode);
        }
    }
    for (const [, edge] of s.edgeLookup) {
        if (edge.selected) {
            selectedEdges.push(edge);
        }
    }
    return { selectedNodes, selectedEdges };
};
const selectId = (obj) => obj.id;
function areEqual(a, b) {
    return (shallow(a.selectedNodes.map(selectId), b.selectedNodes.map(selectId)) &&
        shallow(a.selectedEdges.map(selectId), b.selectedEdges.map(selectId)));
}
function SelectionListenerInner({ onSelectionChange, }) {
    const store = useStoreApi();
    const { selectedNodes, selectedEdges } = useStore(selector$m, areEqual);
    useEffect(() => {
        const params = { nodes: selectedNodes, edges: selectedEdges };
        onSelectionChange?.(params);
        store.getState().onSelectionChangeHandlers.forEach((fn) => fn(params));
    }, [selectedNodes, selectedEdges, onSelectionChange]);
    return null;
}
const changeSelector = (s) => !!s.onSelectionChangeHandlers;
function SelectionListener({ onSelectionChange, }) {
    const storeHasSelectionChangeHandlers = useStore(changeSelector);
    if (onSelectionChange || storeHasSelectionChangeHandlers) {
        return jsx(SelectionListenerInner, { onSelectionChange: onSelectionChange });
    }
    return null;
}

const defaultNodeOrigin = [0, 0];
const defaultViewport = { x: 0, y: 0, zoom: 1 };

/*
 * This component helps us to update the store with the values coming from the user.
 * We distinguish between values we can update directly with `useDirectStoreUpdater` (like `snapGrid`)
 * and values that have a dedicated setter function in the store (like `setNodes`).
 */
// These fields exist in the global store, and we need to keep them up to date
const reactFlowFieldsToTrack = [
    'nodes',
    'edges',
    'defaultNodes',
    'defaultEdges',
    'onConnect',
    'onConnectStart',
    'onConnectEnd',
    'onClickConnectStart',
    'onClickConnectEnd',
    'nodesDraggable',
    'autoPanOnNodeFocus',
    'nodesConnectable',
    'nodesFocusable',
    'edgesFocusable',
    'edgesReconnectable',
    'elevateNodesOnSelect',
    'elevateEdgesOnSelect',
    'minZoom',
    'maxZoom',
    'nodeExtent',
    'onNodesChange',
    'onEdgesChange',
    'elementsSelectable',
    'connectionMode',
    'snapGrid',
    'snapToGrid',
    'translateExtent',
    'connectOnClick',
    'defaultEdgeOptions',
    'fitView',
    'fitViewOptions',
    'onNodesDelete',
    'onEdgesDelete',
    'onDelete',
    'onNodeDrag',
    'onNodeDragStart',
    'onNodeDragStop',
    'onSelectionDrag',
    'onSelectionDragStart',
    'onSelectionDragStop',
    'onMoveStart',
    'onMove',
    'onMoveEnd',
    'noPanClassName',
    'nodeOrigin',
    'autoPanOnConnect',
    'autoPanOnNodeDrag',
    'onError',
    'connectionRadius',
    'isValidConnection',
    'selectNodesOnDrag',
    'nodeDragThreshold',
    'connectionDragThreshold',
    'onBeforeDelete',
    'debug',
    'autoPanSpeed',
    'paneClickDistance',
    'ariaLabelConfig',
];
// rfId doesn't exist in ReactFlowProps, but it's one of the fields we want to update
const fieldsToTrack = [...reactFlowFieldsToTrack, 'rfId'];
const selector$l = (s) => ({
    setNodes: s.setNodes,
    setEdges: s.setEdges,
    setMinZoom: s.setMinZoom,
    setMaxZoom: s.setMaxZoom,
    setTranslateExtent: s.setTranslateExtent,
    setNodeExtent: s.setNodeExtent,
    reset: s.reset,
    setDefaultNodesAndEdges: s.setDefaultNodesAndEdges,
    setPaneClickDistance: s.setPaneClickDistance,
});
const initPrevValues = {
    /*
     * these are values that are also passed directly to other components
     * than the StoreUpdater. We can reduce the number of setStore calls
     * by setting the same values here as prev fields.
     */
    translateExtent: infiniteExtent,
    nodeOrigin: defaultNodeOrigin,
    minZoom: 0.5,
    maxZoom: 2,
    elementsSelectable: true,
    noPanClassName: 'nopan',
    rfId: '1',
    paneClickDistance: 0,
};
function StoreUpdater(props) {
    const { setNodes, setEdges, setMinZoom, setMaxZoom, setTranslateExtent, setNodeExtent, reset, setDefaultNodesAndEdges, setPaneClickDistance, } = useStore(selector$l, shallow);
    const store = useStoreApi();
    useEffect(() => {
        setDefaultNodesAndEdges(props.defaultNodes, props.defaultEdges);
        return () => {
            // when we reset the store we also need to reset the previous fields
            previousFields.current = initPrevValues;
            reset();
        };
    }, []);
    const previousFields = useRef(initPrevValues);
    useEffect(() => {
        for (const fieldName of fieldsToTrack) {
            const fieldValue = props[fieldName];
            const previousFieldValue = previousFields.current[fieldName];
            if (fieldValue === previousFieldValue)
                continue;
            if (typeof props[fieldName] === 'undefined')
                continue;
            // Custom handling with dedicated setters for some fields
            if (fieldName === 'nodes')
                setNodes(fieldValue);
            else if (fieldName === 'edges')
                setEdges(fieldValue);
            else if (fieldName === 'minZoom')
                setMinZoom(fieldValue);
            else if (fieldName === 'maxZoom')
                setMaxZoom(fieldValue);
            else if (fieldName === 'translateExtent')
                setTranslateExtent(fieldValue);
            else if (fieldName === 'nodeExtent')
                setNodeExtent(fieldValue);
            else if (fieldName === 'paneClickDistance')
                setPaneClickDistance(fieldValue);
            else if (fieldName === 'ariaLabelConfig')
                store.setState({ ariaLabelConfig: mergeAriaLabelConfig(fieldValue) });
            // Renamed fields
            else if (fieldName === 'fitView')
                store.setState({ fitViewQueued: fieldValue });
            else if (fieldName === 'fitViewOptions')
                store.setState({ fitViewOptions: fieldValue });
            // General case
            else
                store.setState({ [fieldName]: fieldValue });
        }
        previousFields.current = props;
    }, 
    // Only re-run the effect if one of the fields we track changes
    fieldsToTrack.map((fieldName) => props[fieldName]));
    return null;
}

function getMediaQuery() {
    {
        return null;
    }
}
/**
 * Hook for receiving the current color mode class 'dark' or 'light'.
 *
 * @internal
 * @param colorMode - The color mode to use ('dark', 'light' or 'system')
 */
function useColorModeClass(colorMode) {
    const [colorModeClass, setColorModeClass] = useState(colorMode === 'system' ? null : colorMode);
    useEffect(() => {
        if (colorMode !== 'system') {
            setColorModeClass(colorMode);
            return;
        }
        const mediaQuery = getMediaQuery();
        const updateColorModeClass = () => setColorModeClass(mediaQuery?.matches ? 'dark' : 'light');
        updateColorModeClass();
        mediaQuery?.addEventListener('change', updateColorModeClass);
        return () => {
            mediaQuery?.removeEventListener('change', updateColorModeClass);
        };
    }, [colorMode]);
    return colorModeClass !== null ? colorModeClass : getMediaQuery()?.matches ? 'dark' : 'light';
}

const defaultDoc = typeof document !== 'undefined' ? document : null;
/**
 * This hook lets you listen for specific key codes and tells you whether they are
 * currently pressed or not.
 *
 * @public
 * @param options - Options
 *
 * @example
 * ```tsx
 *import { useKeyPress } from '@xyflow/react';
 *
 *export default function () {
 *  const spacePressed = useKeyPress('Space');
 *  const cmdAndSPressed = useKeyPress(['Meta+s', 'Strg+s']);
 *
 *  return (
 *    <div>
 *     {spacePressed && <p>Space pressed!</p>}
 *     {cmdAndSPressed && <p>Cmd + S pressed!</p>}
 *    </div>
 *  );
 *}
 *```
 */
function useKeyPress(
/**
 * The key code (string or array of strings) specifies which key(s) should trigger
 * an action.
 *
 * A **string** can represent:
 * - A **single key**, e.g. `'a'`
 * - A **key combination**, using `'+'` to separate keys, e.g. `'a+d'`
 *
 * An  **array of strings** represents **multiple possible key inputs**. For example, `['a', 'd+s']`
 * means the user can press either the single key `'a'` or the combination of `'d'` and `'s'`.
 * @default null
 */
keyCode = null, options = { target: defaultDoc, actInsideInputWithModifier: true }) {
    const [keyPressed, setKeyPressed] = useState(false);
    // we need to remember if a modifier key is pressed in order to track it
    const modifierPressed = useRef(false);
    // we need to remember the pressed keys in order to support combinations
    const pressedKeys = useRef(new Set([]));
    /*
     * keyCodes = array with single keys [['a']] or key combinations [['a', 's']]
     * keysToWatch = array with all keys flattened ['a', 'd', 'ShiftLeft']
     * used to check if we store event.code or event.key. When the code is in the list of keysToWatch
     * we use the code otherwise the key. Explainer: When you press the left "command" key, the code is "MetaLeft"
     * and the key is "Meta". We want users to be able to pass keys and codes so we assume that the key is meant when
     * we can't find it in the list of keysToWatch.
     */
    const [keyCodes, keysToWatch] = useMemo(() => {
        if (keyCode !== null) {
            const keyCodeArr = Array.isArray(keyCode) ? keyCode : [keyCode];
            const keys = keyCodeArr
                .filter((kc) => typeof kc === 'string')
                /*
                 * we first replace all '+' with '\n'  which we will use to split the keys on
                 * then we replace '\n\n' with '\n+', this way we can also support the combination 'key++'
                 * in the end we simply split on '\n' to get the key array
                 */
                .map((kc) => kc.replace('+', '\n').replace('\n\n', '\n+').split('\n'));
            const keysFlat = keys.reduce((res, item) => res.concat(...item), []);
            return [keys, keysFlat];
        }
        return [[], []];
    }, [keyCode]);
    useEffect(() => {
        const target = options?.target ?? defaultDoc;
        const actInsideInputWithModifier = options?.actInsideInputWithModifier ?? true;
        if (keyCode !== null) {
            const downHandler = (event) => {
                modifierPressed.current = event.ctrlKey || event.metaKey || event.shiftKey || event.altKey;
                const preventAction = (!modifierPressed.current || (modifierPressed.current && !actInsideInputWithModifier)) &&
                    isInputDOMNode(event);
                if (preventAction) {
                    return false;
                }
                const keyOrCode = useKeyOrCode(event.code, keysToWatch);
                pressedKeys.current.add(event[keyOrCode]);
                if (isMatchingKey(keyCodes, pressedKeys.current, false)) {
                    const target = (event.composedPath?.()?.[0] || event.target);
                    const isInteractiveElement = target?.nodeName === 'BUTTON' || target?.nodeName === 'A';
                    if (options.preventDefault !== false && (modifierPressed.current || !isInteractiveElement)) {
                        event.preventDefault();
                    }
                    setKeyPressed(true);
                }
            };
            const upHandler = (event) => {
                const keyOrCode = useKeyOrCode(event.code, keysToWatch);
                if (isMatchingKey(keyCodes, pressedKeys.current, true)) {
                    setKeyPressed(false);
                    pressedKeys.current.clear();
                }
                else {
                    pressedKeys.current.delete(event[keyOrCode]);
                }
                // fix for Mac: when cmd key is pressed, keyup is not triggered for any other key, see: https://stackoverflow.com/questions/27380018/when-cmd-key-is-kept-pressed-keyup-is-not-triggered-for-any-other-key
                if (event.key === 'Meta') {
                    pressedKeys.current.clear();
                }
                modifierPressed.current = false;
            };
            const resetHandler = () => {
                pressedKeys.current.clear();
                setKeyPressed(false);
            };
            target?.addEventListener('keydown', downHandler);
            target?.addEventListener('keyup', upHandler);
            window.addEventListener('blur', resetHandler);
            window.addEventListener('contextmenu', resetHandler);
            return () => {
                target?.removeEventListener('keydown', downHandler);
                target?.removeEventListener('keyup', upHandler);
                window.removeEventListener('blur', resetHandler);
                window.removeEventListener('contextmenu', resetHandler);
            };
        }
    }, [keyCode, setKeyPressed]);
    return keyPressed;
}
// utils
function isMatchingKey(keyCodes, pressedKeys, isUp) {
    return (keyCodes
        /*
         * we only want to compare same sizes of keyCode definitions
         * and pressed keys. When the user specified 'Meta' as a key somewhere
         * this would also be truthy without this filter when user presses 'Meta' + 'r'
         */
        .filter((keys) => isUp || keys.length === pressedKeys.size)
        /*
         * since we want to support multiple possibilities only one of the
         * combinations need to be part of the pressed keys
         */
        .some((keys) => keys.every((k) => pressedKeys.has(k))));
}
function useKeyOrCode(eventCode, keysToWatch) {
    return keysToWatch.includes(eventCode) ? 'code' : 'key';
}

/**
 * Hook for getting viewport helper functions.
 *
 * @internal
 * @returns viewport helper functions
 */
const useViewportHelper = () => {
    const store = useStoreApi();
    return useMemo(() => {
        return {
            zoomIn: (options) => {
                const { panZoom } = store.getState();
                return panZoom ? panZoom.scaleBy(1.2, { duration: options?.duration }) : Promise.resolve(false);
            },
            zoomOut: (options) => {
                const { panZoom } = store.getState();
                return panZoom ? panZoom.scaleBy(1 / 1.2, { duration: options?.duration }) : Promise.resolve(false);
            },
            zoomTo: (zoomLevel, options) => {
                const { panZoom } = store.getState();
                return panZoom ? panZoom.scaleTo(zoomLevel, { duration: options?.duration }) : Promise.resolve(false);
            },
            getZoom: () => store.getState().transform[2],
            setViewport: async (viewport, options) => {
                const { transform: [tX, tY, tZoom], panZoom, } = store.getState();
                if (!panZoom) {
                    return Promise.resolve(false);
                }
                await panZoom.setViewport({
                    x: viewport.x ?? tX,
                    y: viewport.y ?? tY,
                    zoom: viewport.zoom ?? tZoom,
                }, options);
                return Promise.resolve(true);
            },
            getViewport: () => {
                const [x, y, zoom] = store.getState().transform;
                return { x, y, zoom };
            },
            setCenter: async (x, y, options) => {
                return store.getState().setCenter(x, y, options);
            },
            fitBounds: async (bounds, options) => {
                const { width, height, minZoom, maxZoom, panZoom } = store.getState();
                const viewport = getViewportForBounds(bounds, width, height, minZoom, maxZoom, options?.padding ?? 0.1);
                if (!panZoom) {
                    return Promise.resolve(false);
                }
                await panZoom.setViewport(viewport, {
                    duration: options?.duration,
                    ease: options?.ease,
                    interpolate: options?.interpolate,
                });
                return Promise.resolve(true);
            },
            screenToFlowPosition: (clientPosition, options = {}) => {
                const { transform, snapGrid, snapToGrid, domNode } = store.getState();
                if (!domNode) {
                    return clientPosition;
                }
                const { x: domX, y: domY } = domNode.getBoundingClientRect();
                const correctedPosition = {
                    x: clientPosition.x - domX,
                    y: clientPosition.y - domY,
                };
                const _snapGrid = options.snapGrid ?? snapGrid;
                const _snapToGrid = options.snapToGrid ?? snapToGrid;
                return pointToRendererPoint(correctedPosition, transform, _snapToGrid, _snapGrid);
            },
            flowToScreenPosition: (flowPosition) => {
                const { transform, domNode } = store.getState();
                if (!domNode) {
                    return flowPosition;
                }
                const { x: domX, y: domY } = domNode.getBoundingClientRect();
                const rendererPosition = rendererPointToPoint(flowPosition, transform);
                return {
                    x: rendererPosition.x + domX,
                    y: rendererPosition.y + domY,
                };
            },
        };
    }, []);
};

/*
 * This function applies changes to nodes or edges that are triggered by React Flow internally.
 * When you drag a node for example, React Flow will send a position change update.
 * This function then applies the changes and returns the updated elements.
 */
function applyChanges(changes, elements) {
    const updatedElements = [];
    /*
     * By storing a map of changes for each element, we can a quick lookup as we
     * iterate over the elements array!
     */
    const changesMap = new Map();
    const addItemChanges = [];
    for (const change of changes) {
        if (change.type === 'add') {
            addItemChanges.push(change);
            continue;
        }
        else if (change.type === 'remove' || change.type === 'replace') {
            /*
             * For a 'remove' change we can safely ignore any other changes queued for
             * the same element, it's going to be removed anyway!
             */
            changesMap.set(change.id, [change]);
        }
        else {
            const elementChanges = changesMap.get(change.id);
            if (elementChanges) {
                /*
                 * If we have some changes queued already, we can do a mutable update of
                 * that array and save ourselves some copying.
                 */
                elementChanges.push(change);
            }
            else {
                changesMap.set(change.id, [change]);
            }
        }
    }
    for (const element of elements) {
        const changes = changesMap.get(element.id);
        /*
         * When there are no changes for an element we can just push it unmodified,
         * no need to copy it.
         */
        if (!changes) {
            updatedElements.push(element);
            continue;
        }
        // If we have a 'remove' change queued, it'll be the only change in the array
        if (changes[0].type === 'remove') {
            continue;
        }
        if (changes[0].type === 'replace') {
            updatedElements.push({ ...changes[0].item });
            continue;
        }
        /**
         * For other types of changes, we want to start with a shallow copy of the
         * object so React knows this element has changed. Sequential changes will
         * each _mutate_ this object, so there's only ever one copy.
         */
        const updatedElement = { ...element };
        for (const change of changes) {
            applyChange(change, updatedElement);
        }
        updatedElements.push(updatedElement);
    }
    /*
     * we need to wait for all changes to be applied before adding new items
     * to be able to add them at the correct index
     */
    if (addItemChanges.length) {
        addItemChanges.forEach((change) => {
            if (change.index !== undefined) {
                updatedElements.splice(change.index, 0, { ...change.item });
            }
            else {
                updatedElements.push({ ...change.item });
            }
        });
    }
    return updatedElements;
}
// Applies a single change to an element. This is a *mutable* update.
function applyChange(change, element) {
    switch (change.type) {
        case 'select': {
            element.selected = change.selected;
            break;
        }
        case 'position': {
            if (typeof change.position !== 'undefined') {
                element.position = change.position;
            }
            if (typeof change.dragging !== 'undefined') {
                element.dragging = change.dragging;
            }
            break;
        }
        case 'dimensions': {
            if (typeof change.dimensions !== 'undefined') {
                element.measured ??= {};
                element.measured.width = change.dimensions.width;
                element.measured.height = change.dimensions.height;
                if (change.setAttributes) {
                    if (change.setAttributes === true || change.setAttributes === 'width') {
                        element.width = change.dimensions.width;
                    }
                    if (change.setAttributes === true || change.setAttributes === 'height') {
                        element.height = change.dimensions.height;
                    }
                }
            }
            if (typeof change.resizing === 'boolean') {
                element.resizing = change.resizing;
            }
            break;
        }
    }
}
/**
 * Drop in function that applies node changes to an array of nodes.
 * @public
 * @param changes - Array of changes to apply.
 * @param nodes - Array of nodes to apply the changes to.
 * @returns Array of updated nodes.
 * @example
 *```tsx
 *import { useState, useCallback } from 'react';
 *import { ReactFlow, applyNodeChanges, type Node, type Edge, type OnNodesChange } from '@xyflow/react';
 *
 *export default function Flow() {
 *  const [nodes, setNodes] = useState<Node[]>([]);
 *  const [edges, setEdges] = useState<Edge[]>([]);
 *  const onNodesChange: OnNodesChange = useCallback(
 *    (changes) => {
 *      setNodes((oldNodes) => applyNodeChanges(changes, oldNodes));
 *    },
 *    [setNodes],
 *  );
 *
 *  return (
 *    <ReactFlow nodes={nodes} edges={edges} onNodesChange={onNodesChange} />
 *  );
 *}
 *```
 * @remarks Various events on the <ReactFlow /> component can produce an {@link NodeChange}
 * that describes how to update the edges of your flow in some way.
 * If you don't need any custom behaviour, this util can be used to take an array
 * of these changes and apply them to your edges.
 */
function applyNodeChanges(changes, nodes) {
    return applyChanges(changes, nodes);
}
/**
 * Drop in function that applies edge changes to an array of edges.
 * @public
 * @param changes - Array of changes to apply.
 * @param edges - Array of edge to apply the changes to.
 * @returns Array of updated edges.
 * @example
 * ```tsx
 *import { useState, useCallback } from 'react';
 *import { ReactFlow, applyEdgeChanges } from '@xyflow/react';
 *
 *export default function Flow() {
 *  const [nodes, setNodes] = useState([]);
 *  const [edges, setEdges] = useState([]);
 *  const onEdgesChange = useCallback(
 *    (changes) => {
 *      setEdges((oldEdges) => applyEdgeChanges(changes, oldEdges));
 *    },
 *    [setEdges],
 *  );
 *
 *  return (
 *    <ReactFlow nodes={nodes} edges={edges} onEdgesChange={onEdgesChange} />
 *  );
 *}
 *```
 * @remarks Various events on the <ReactFlow /> component can produce an {@link EdgeChange}
 * that describes how to update the edges of your flow in some way.
 * If you don't need any custom behaviour, this util can be used to take an array
 * of these changes and apply them to your edges.
 */
function applyEdgeChanges(changes, edges) {
    return applyChanges(changes, edges);
}
function createSelectionChange(id, selected) {
    return {
        id,
        type: 'select',
        selected,
    };
}
function getSelectionChanges(items, selectedIds = new Set(), mutateItem = false) {
    const changes = [];
    for (const [id, item] of items) {
        const willBeSelected = selectedIds.has(id);
        // we don't want to set all items to selected=false on the first selection
        if (!(item.selected === undefined && !willBeSelected) && item.selected !== willBeSelected) {
            if (mutateItem) {
                /*
                 * this hack is needed for nodes. When the user dragged a node, it's selected.
                 * When another node gets dragged, we need to deselect the previous one,
                 * in order to have only one selected node at a time - the onNodesChange callback comes too late here :/
                 */
                item.selected = willBeSelected;
            }
            changes.push(createSelectionChange(item.id, willBeSelected));
        }
    }
    return changes;
}
function getElementsDiffChanges({ items = [], lookup, }) {
    const changes = [];
    const itemsLookup = new Map(items.map((item) => [item.id, item]));
    for (const [index, item] of items.entries()) {
        const lookupItem = lookup.get(item.id);
        const storeItem = lookupItem?.internals?.userNode ?? lookupItem;
        if (storeItem !== undefined && storeItem !== item) {
            changes.push({ id: item.id, item: item, type: 'replace' });
        }
        if (storeItem === undefined) {
            changes.push({ item: item, type: 'add', index });
        }
    }
    for (const [id] of lookup) {
        const nextNode = itemsLookup.get(id);
        if (nextNode === undefined) {
            changes.push({ id, type: 'remove' });
        }
    }
    return changes;
}
function elementToRemoveChange(item) {
    return {
        id: item.id,
        type: 'remove',
    };
}

/**
 * Test whether an object is usable as an [`Node`](/api-reference/types/node).
 * In TypeScript this is a type guard that will narrow the type of whatever you pass in to
 * [`Node`](/api-reference/types/node) if it returns `true`.
 *
 * @public
 * @remarks In TypeScript this is a type guard that will narrow the type of whatever you pass in to Node if it returns true
 * @param element - The element to test.
 * @returns Tests whether the provided value can be used as a `Node`. If you're using TypeScript,
 * this function acts as a type guard and will narrow the type of the value to `Node` if it returns
 * `true`.
 *
 * @example
 * ```js
 *import { isNode } from '@xyflow/react';
 *
 *if (isNode(node)) {
 * // ...
 *}
 *```
 */
const isNode = (element) => isNodeBase(element);
/**
 * Test whether an object is usable as an [`Edge`](/api-reference/types/edge).
 * In TypeScript this is a type guard that will narrow the type of whatever you pass in to
 * [`Edge`](/api-reference/types/edge) if it returns `true`.
 *
 * @public
 * @remarks In TypeScript this is a type guard that will narrow the type of whatever you pass in to Edge if it returns true
 * @param element - The element to test
 * @returns Tests whether the provided value can be used as an `Edge`. If you're using TypeScript,
 * this function acts as a type guard and will narrow the type of the value to `Edge` if it returns
 * `true`.
 *
 * @example
 * ```js
 *import { isEdge } from '@xyflow/react';
 *
 *if (isEdge(edge)) {
 * // ...
 *}
 *```
 */
const isEdge = (element) => isEdgeBase(element);
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
function fixedForwardRef(render) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return forwardRef(render);
}

// we need this hook to prevent a warning when using react-flow in SSR
const useIsomorphicLayoutEffect = useEffect;

/**
 * This hook returns a queue that can be used to batch updates.
 *
 * @param runQueue - a function that gets called when the queue is flushed
 * @internal
 *
 * @returns a Queue object
 */
function useQueue(runQueue) {
    /*
     * Because we're using a ref above, we need some way to let React know when to
     * actually process the queue. We increment this number any time we mutate the
     * queue, creating a new state to trigger the layout effect below.
     * Using a boolean dirty flag here instead would lead to issues related to
     * automatic batching. (https://github.com/xyflow/xyflow/issues/4779)
     */
    const [serial, setSerial] = useState(BigInt(0));
    /*
     * A reference of all the batched updates to process before the next render. We
     * want a reference here so multiple synchronous calls to `setNodes` etc can be
     * batched together.
     */
    const [queue] = useState(() => createQueue(() => setSerial(n => n + BigInt(1))));
    /*
     * Layout effects are guaranteed to run before the next render which means we
     * shouldn't run into any issues with stale state or weird issues that come from
     * rendering things one frame later than expected (we used to use `setTimeout`).
     */
    useIsomorphicLayoutEffect(() => {
        const queueItems = queue.get();
        if (queueItems.length) {
            runQueue(queueItems);
            queue.reset();
        }
    }, [serial]);
    return queue;
}
function createQueue(cb) {
    let queue = [];
    return {
        get: () => queue,
        reset: () => {
            queue = [];
        },
        push: (item) => {
            queue.push(item);
            cb();
        },
    };
}

const BatchContext = createContext(null);
/**
 * This is a context provider that holds and processes the node and edge update queues
 * that are needed to handle setNodes, addNodes, setEdges and addEdges.
 *
 * @internal
 */
function BatchProvider({ children, }) {
    const store = useStoreApi();
    const nodeQueueHandler = useCallback((queueItems) => {
        const { nodes = [], setNodes, hasDefaultNodes, onNodesChange, nodeLookup, fitViewQueued } = store.getState();
        /*
         * This is essentially an `Array.reduce` in imperative clothing. Processing
         * this queue is a relatively hot path so we'd like to avoid the overhead of
         * array methods where we can.
         */
        let next = nodes;
        for (const payload of queueItems) {
            next = typeof payload === 'function' ? payload(next) : payload;
        }
        const changes = getElementsDiffChanges({
            items: next,
            lookup: nodeLookup,
        });
        if (hasDefaultNodes) {
            setNodes(next);
        }
        // We only want to fire onNodesChange if there are changes to the nodes
        if (changes.length > 0) {
            onNodesChange?.(changes);
        }
        else if (fitViewQueued) {
            // If there are no changes to the nodes, we still need to call setNodes
            // to trigger a re-render and fitView.
            window.requestAnimationFrame(() => {
                const { fitViewQueued, nodes, setNodes } = store.getState();
                if (fitViewQueued) {
                    setNodes(nodes);
                }
            });
        }
    }, []);
    const nodeQueue = useQueue(nodeQueueHandler);
    const edgeQueueHandler = useCallback((queueItems) => {
        const { edges = [], setEdges, hasDefaultEdges, onEdgesChange, edgeLookup } = store.getState();
        let next = edges;
        for (const payload of queueItems) {
            next = typeof payload === 'function' ? payload(next) : payload;
        }
        if (hasDefaultEdges) {
            setEdges(next);
        }
        else if (onEdgesChange) {
            onEdgesChange(getElementsDiffChanges({
                items: next,
                lookup: edgeLookup,
            }));
        }
    }, []);
    const edgeQueue = useQueue(edgeQueueHandler);
    const value = useMemo(() => ({ nodeQueue, edgeQueue }), []);
    return jsx(BatchContext.Provider, { value: value, children: children });
}
function useBatchContext() {
    const batchContext = useContext(BatchContext);
    if (!batchContext) {
        throw new Error('useBatchContext must be used within a BatchProvider');
    }
    return batchContext;
}

const selector$k = (s) => !!s.panZoom;
/**
 * This hook returns a ReactFlowInstance that can be used to update nodes and edges, manipulate the viewport, or query the current state of the flow.
 *
 * @public
 * @example
 * ```jsx
 *import { useCallback, useState } from 'react';
 *import { useReactFlow } from '@xyflow/react';
 *
 *export function NodeCounter() {
 *  const reactFlow = useReactFlow();
 *  const [count, setCount] = useState(0);
 *  const countNodes = useCallback(() => {
 *    setCount(reactFlow.getNodes().length);
 *    // you need to pass it as a dependency if you are using it with useEffect or useCallback
 *    // because at the first render, it's not initialized yet and some functions might not work.
 *  }, [reactFlow]);
 *
 *  return (
 *    <div>
 *      <button onClick={countNodes}>Update count</button>
 *      <p>There are {count} nodes in the flow.</p>
 *    </div>
 *  );
 *}
 *```
 */
function useReactFlow() {
    const viewportHelper = useViewportHelper();
    const store = useStoreApi();
    const batchContext = useBatchContext();
    const viewportInitialized = useStore(selector$k);
    const generalHelper = useMemo(() => {
        const getInternalNode = (id) => store.getState().nodeLookup.get(id);
        const setNodes = (payload) => {
            batchContext.nodeQueue.push(payload);
        };
        const setEdges = (payload) => {
            batchContext.edgeQueue.push(payload);
        };
        const getNodeRect = (node) => {
            const { nodeLookup, nodeOrigin } = store.getState();
            const nodeToUse = isNode(node) ? node : nodeLookup.get(node.id);
            const position = nodeToUse.parentId
                ? evaluateAbsolutePosition(nodeToUse.position, nodeToUse.measured, nodeToUse.parentId, nodeLookup, nodeOrigin)
                : nodeToUse.position;
            const nodeWithPosition = {
                ...nodeToUse,
                position,
                width: nodeToUse.measured?.width ?? nodeToUse.width,
                height: nodeToUse.measured?.height ?? nodeToUse.height,
            };
            return nodeToRect(nodeWithPosition);
        };
        const updateNode = (id, nodeUpdate, options = { replace: false }) => {
            setNodes((prevNodes) => prevNodes.map((node) => {
                if (node.id === id) {
                    const nextNode = typeof nodeUpdate === 'function' ? nodeUpdate(node) : nodeUpdate;
                    return options.replace && isNode(nextNode) ? nextNode : { ...node, ...nextNode };
                }
                return node;
            }));
        };
        const updateEdge = (id, edgeUpdate, options = { replace: false }) => {
            setEdges((prevEdges) => prevEdges.map((edge) => {
                if (edge.id === id) {
                    const nextEdge = typeof edgeUpdate === 'function' ? edgeUpdate(edge) : edgeUpdate;
                    return options.replace && isEdge(nextEdge) ? nextEdge : { ...edge, ...nextEdge };
                }
                return edge;
            }));
        };
        return {
            getNodes: () => store.getState().nodes.map((n) => ({ ...n })),
            getNode: (id) => getInternalNode(id)?.internals.userNode,
            getInternalNode,
            getEdges: () => {
                const { edges = [] } = store.getState();
                return edges.map((e) => ({ ...e }));
            },
            getEdge: (id) => store.getState().edgeLookup.get(id),
            setNodes,
            setEdges,
            addNodes: (payload) => {
                const newNodes = Array.isArray(payload) ? payload : [payload];
                batchContext.nodeQueue.push((nodes) => [...nodes, ...newNodes]);
            },
            addEdges: (payload) => {
                const newEdges = Array.isArray(payload) ? payload : [payload];
                batchContext.edgeQueue.push((edges) => [...edges, ...newEdges]);
            },
            toObject: () => {
                const { nodes = [], edges = [], transform } = store.getState();
                const [x, y, zoom] = transform;
                return {
                    nodes: nodes.map((n) => ({ ...n })),
                    edges: edges.map((e) => ({ ...e })),
                    viewport: {
                        x,
                        y,
                        zoom,
                    },
                };
            },
            deleteElements: async ({ nodes: nodesToRemove = [], edges: edgesToRemove = [] }) => {
                const { nodes, edges, onNodesDelete, onEdgesDelete, triggerNodeChanges, triggerEdgeChanges, onDelete, onBeforeDelete, } = store.getState();
                const { nodes: matchingNodes, edges: matchingEdges } = await getElementsToRemove({
                    nodesToRemove,
                    edgesToRemove,
                    nodes,
                    edges,
                    onBeforeDelete,
                });
                const hasMatchingEdges = matchingEdges.length > 0;
                const hasMatchingNodes = matchingNodes.length > 0;
                if (hasMatchingEdges) {
                    const edgeChanges = matchingEdges.map(elementToRemoveChange);
                    onEdgesDelete?.(matchingEdges);
                    triggerEdgeChanges(edgeChanges);
                }
                if (hasMatchingNodes) {
                    const nodeChanges = matchingNodes.map(elementToRemoveChange);
                    onNodesDelete?.(matchingNodes);
                    triggerNodeChanges(nodeChanges);
                }
                if (hasMatchingNodes || hasMatchingEdges) {
                    onDelete?.({ nodes: matchingNodes, edges: matchingEdges });
                }
                return { deletedNodes: matchingNodes, deletedEdges: matchingEdges };
            },
            getIntersectingNodes: (nodeOrRect, partially = true, nodes) => {
                const isRect = isRectObject(nodeOrRect);
                const nodeRect = isRect ? nodeOrRect : getNodeRect(nodeOrRect);
                const hasNodesOption = nodes !== undefined;
                if (!nodeRect) {
                    return [];
                }
                return (nodes || store.getState().nodes).filter((n) => {
                    const internalNode = store.getState().nodeLookup.get(n.id);
                    if (internalNode && !isRect && (n.id === nodeOrRect.id || !internalNode.internals.positionAbsolute)) {
                        return false;
                    }
                    const currNodeRect = nodeToRect(hasNodesOption ? n : internalNode);
                    const overlappingArea = getOverlappingArea(currNodeRect, nodeRect);
                    const partiallyVisible = partially && overlappingArea > 0;
                    return (partiallyVisible ||
                        overlappingArea >= currNodeRect.width * currNodeRect.height ||
                        overlappingArea >= nodeRect.width * nodeRect.height);
                });
            },
            isNodeIntersecting: (nodeOrRect, area, partially = true) => {
                const isRect = isRectObject(nodeOrRect);
                const nodeRect = isRect ? nodeOrRect : getNodeRect(nodeOrRect);
                if (!nodeRect) {
                    return false;
                }
                const overlappingArea = getOverlappingArea(nodeRect, area);
                const partiallyVisible = partially && overlappingArea > 0;
                return partiallyVisible || overlappingArea >= nodeRect.width * nodeRect.height;
            },
            updateNode,
            updateNodeData: (id, dataUpdate, options = { replace: false }) => {
                updateNode(id, (node) => {
                    const nextData = typeof dataUpdate === 'function' ? dataUpdate(node) : dataUpdate;
                    return options.replace ? { ...node, data: nextData } : { ...node, data: { ...node.data, ...nextData } };
                }, options);
            },
            updateEdge,
            updateEdgeData: (id, dataUpdate, options = { replace: false }) => {
                updateEdge(id, (edge) => {
                    const nextData = typeof dataUpdate === 'function' ? dataUpdate(edge) : dataUpdate;
                    return options.replace ? { ...edge, data: nextData } : { ...edge, data: { ...edge.data, ...nextData } };
                }, options);
            },
            getNodesBounds: (nodes) => {
                const { nodeLookup, nodeOrigin } = store.getState();
                return getNodesBounds(nodes, { nodeLookup, nodeOrigin });
            },
            getHandleConnections: ({ type, id, nodeId }) => Array.from(store
                .getState()
                .connectionLookup.get(`${nodeId}-${type}${id ? `-${id}` : ''}`)
                ?.values() ?? []),
            getNodeConnections: ({ type, handleId, nodeId }) => Array.from(store
                .getState()
                .connectionLookup.get(`${nodeId}${type ? (handleId ? `-${type}-${handleId}` : `-${type}`) : ''}`)
                ?.values() ?? []),
            fitView: async (options) => {
                // We either create a new Promise or reuse the existing one
                // Even if fitView is called multiple times in a row, we only end up with a single Promise
                const fitViewResolver = store.getState().fitViewResolver ?? withResolvers();
                // We schedule a fitView by setting fitViewQueued and triggering a setNodes
                store.setState({ fitViewQueued: true, fitViewOptions: options, fitViewResolver });
                batchContext.nodeQueue.push((nodes) => [...nodes]);
                return fitViewResolver.promise;
            },
        };
    }, []);
    return useMemo(() => {
        return {
            ...generalHelper,
            ...viewportHelper,
            viewportInitialized,
        };
    }, [viewportInitialized]);
}

const selected = (item) => item.selected;
const win$1 = undefined;
/**
 * Hook for handling global key events.
 *
 * @internal
 */
function useGlobalKeyHandler({ deleteKeyCode, multiSelectionKeyCode, }) {
    const store = useStoreApi();
    const { deleteElements } = useReactFlow();
    const deleteKeyPressed = useKeyPress(deleteKeyCode, { actInsideInputWithModifier: false });
    const multiSelectionKeyPressed = useKeyPress(multiSelectionKeyCode, { target: win$1 });
    useEffect(() => {
        if (deleteKeyPressed) {
            const { edges, nodes } = store.getState();
            deleteElements({ nodes: nodes.filter(selected), edges: edges.filter(selected) });
            store.setState({ nodesSelectionActive: false });
        }
    }, [deleteKeyPressed]);
    useEffect(() => {
        store.setState({ multiSelectionActive: multiSelectionKeyPressed });
    }, [multiSelectionKeyPressed]);
}

/**
 * Hook for handling resize events.
 *
 * @internal
 */
function useResizeHandler(domNode) {
    const store = useStoreApi();
    useEffect(() => {
        const updateDimensions = () => {
            if (!domNode.current) {
                return false;
            }
            const size = getDimensions(domNode.current);
            if (size.height === 0 || size.width === 0) {
                store.getState().onError?.('004', errorMessages['error004']());
            }
            store.setState({ width: size.width || 500, height: size.height || 500 });
        };
        if (domNode.current) {
            updateDimensions();
            window.addEventListener('resize', updateDimensions);
            const resizeObserver = new ResizeObserver(() => updateDimensions());
            resizeObserver.observe(domNode.current);
            return () => {
                window.removeEventListener('resize', updateDimensions);
                if (resizeObserver && domNode.current) {
                    resizeObserver.unobserve(domNode.current);
                }
            };
        }
    }, []);
}

const containerStyle = {
    position: 'absolute',
    width: '100%',
    height: '100%',
    top: 0,
    left: 0,
};

const selector$j = (s) => ({
    userSelectionActive: s.userSelectionActive,
    lib: s.lib,
});
function ZoomPane({ onPaneContextMenu, zoomOnScroll = true, zoomOnPinch = true, panOnScroll = false, panOnScrollSpeed = 0.5, panOnScrollMode = PanOnScrollMode.Free, zoomOnDoubleClick = true, panOnDrag = true, defaultViewport, translateExtent, minZoom, maxZoom, zoomActivationKeyCode, preventScrolling = true, children, noWheelClassName, noPanClassName, onViewportChange, isControlledViewport, paneClickDistance, }) {
    const store = useStoreApi();
    const zoomPane = useRef(null);
    const { userSelectionActive, lib } = useStore(selector$j, shallow);
    const zoomActivationKeyPressed = useKeyPress(zoomActivationKeyCode);
    const panZoom = useRef();
    useResizeHandler(zoomPane);
    const onTransformChange = useCallback((transform) => {
        onViewportChange?.({ x: transform[0], y: transform[1], zoom: transform[2] });
        if (!isControlledViewport) {
            store.setState({ transform });
        }
    }, [onViewportChange, isControlledViewport]);
    useEffect(() => {
        if (zoomPane.current) {
            panZoom.current = XYPanZoom({
                domNode: zoomPane.current,
                minZoom,
                maxZoom,
                translateExtent,
                viewport: defaultViewport,
                paneClickDistance,
                onDraggingChange: (paneDragging) => store.setState({ paneDragging }),
                onPanZoomStart: (event, vp) => {
                    const { onViewportChangeStart, onMoveStart } = store.getState();
                    onMoveStart?.(event, vp);
                    onViewportChangeStart?.(vp);
                },
                onPanZoom: (event, vp) => {
                    const { onViewportChange, onMove } = store.getState();
                    onMove?.(event, vp);
                    onViewportChange?.(vp);
                },
                onPanZoomEnd: (event, vp) => {
                    const { onViewportChangeEnd, onMoveEnd } = store.getState();
                    onMoveEnd?.(event, vp);
                    onViewportChangeEnd?.(vp);
                },
            });
            const { x, y, zoom } = panZoom.current.getViewport();
            store.setState({
                panZoom: panZoom.current,
                transform: [x, y, zoom],
                domNode: zoomPane.current.closest('.react-flow'),
            });
            return () => {
                panZoom.current?.destroy();
            };
        }
    }, []);
    useEffect(() => {
        panZoom.current?.update({
            onPaneContextMenu,
            zoomOnScroll,
            zoomOnPinch,
            panOnScroll,
            panOnScrollSpeed,
            panOnScrollMode,
            zoomOnDoubleClick,
            panOnDrag,
            zoomActivationKeyPressed,
            preventScrolling,
            noPanClassName,
            userSelectionActive,
            noWheelClassName,
            lib,
            onTransformChange,
        });
    }, [
        onPaneContextMenu,
        zoomOnScroll,
        zoomOnPinch,
        panOnScroll,
        panOnScrollSpeed,
        panOnScrollMode,
        zoomOnDoubleClick,
        panOnDrag,
        zoomActivationKeyPressed,
        preventScrolling,
        noPanClassName,
        userSelectionActive,
        noWheelClassName,
        lib,
        onTransformChange,
    ]);
    return (jsx("div", { className: "react-flow__renderer", ref: zoomPane, style: containerStyle, children: children }));
}

const selector$i = (s) => ({
    userSelectionActive: s.userSelectionActive,
    userSelectionRect: s.userSelectionRect,
});
function UserSelection() {
    const { userSelectionActive, userSelectionRect } = useStore(selector$i, shallow);
    const isActive = userSelectionActive && userSelectionRect;
    if (!isActive) {
        return null;
    }
    return (jsx("div", { className: "react-flow__selection react-flow__container", style: {
            width: userSelectionRect.width,
            height: userSelectionRect.height,
            transform: `translate(${userSelectionRect.x}px, ${userSelectionRect.y}px)`,
        } }));
}

const wrapHandler = (handler, containerRef) => {
    return (event) => {
        if (event.target !== containerRef.current) {
            return;
        }
        handler?.(event);
    };
};
const selector$h = (s) => ({
    userSelectionActive: s.userSelectionActive,
    elementsSelectable: s.elementsSelectable,
    connectionInProgress: s.connection.inProgress,
    dragging: s.paneDragging,
});
function Pane({ isSelecting, selectionKeyPressed, selectionMode = SelectionMode.Full, panOnDrag, selectionOnDrag, onSelectionStart, onSelectionEnd, onPaneClick, onPaneContextMenu, onPaneScroll, onPaneMouseEnter, onPaneMouseMove, onPaneMouseLeave, children, }) {
    const store = useStoreApi();
    const { userSelectionActive, elementsSelectable, dragging, connectionInProgress } = useStore(selector$h, shallow);
    const hasActiveSelection = elementsSelectable && (isSelecting || userSelectionActive);
    const container = useRef(null);
    const containerBounds = useRef();
    const selectedNodeIds = useRef(new Set());
    const selectedEdgeIds = useRef(new Set());
    // Used to prevent click events when the user lets go of the selectionKey during a selection
    const selectionInProgress = useRef(false);
    const selectionStarted = useRef(false);
    const onClick = (event) => {
        // We prevent click events when the user let go of the selectionKey during a selection
        // We also prevent click events when a connection is in progress
        if (selectionInProgress.current || connectionInProgress) {
            selectionInProgress.current = false;
            return;
        }
        onPaneClick?.(event);
        store.getState().resetSelectedElements();
        store.setState({ nodesSelectionActive: false });
    };
    const onContextMenu = (event) => {
        if (Array.isArray(panOnDrag) && panOnDrag?.includes(2)) {
            event.preventDefault();
            return;
        }
        onPaneContextMenu?.(event);
    };
    const onWheel = onPaneScroll ? (event) => onPaneScroll(event) : undefined;
    const onPointerDown = (event) => {
        const { resetSelectedElements, domNode } = store.getState();
        containerBounds.current = domNode?.getBoundingClientRect();
        if (!elementsSelectable ||
            !isSelecting ||
            event.button !== 0 ||
            event.target !== container.current ||
            !containerBounds.current) {
            return;
        }
        event.target?.setPointerCapture?.(event.pointerId);
        selectionStarted.current = true;
        selectionInProgress.current = false;
        const { x, y } = getEventPosition(event.nativeEvent, containerBounds.current);
        resetSelectedElements();
        store.setState({
            userSelectionRect: {
                width: 0,
                height: 0,
                startX: x,
                startY: y,
                x,
                y,
            },
        });
        onSelectionStart?.(event);
    };
    const onPointerMove = (event) => {
        const { userSelectionRect, transform, nodeLookup, edgeLookup, connectionLookup, triggerNodeChanges, triggerEdgeChanges, defaultEdgeOptions, } = store.getState();
        if (!containerBounds.current || !userSelectionRect) {
            return;
        }
        selectionInProgress.current = true;
        const { x: mouseX, y: mouseY } = getEventPosition(event.nativeEvent, containerBounds.current);
        const { startX, startY } = userSelectionRect;
        const nextUserSelectRect = {
            startX,
            startY,
            x: mouseX < startX ? mouseX : startX,
            y: mouseY < startY ? mouseY : startY,
            width: Math.abs(mouseX - startX),
            height: Math.abs(mouseY - startY),
        };
        const prevSelectedNodeIds = selectedNodeIds.current;
        const prevSelectedEdgeIds = selectedEdgeIds.current;
        selectedNodeIds.current = new Set(getNodesInside(nodeLookup, nextUserSelectRect, transform, selectionMode === SelectionMode.Partial, true).map((node) => node.id));
        selectedEdgeIds.current = new Set();
        const edgesSelectable = defaultEdgeOptions?.selectable ?? true;
        // We look for all edges connected to the selected nodes
        for (const nodeId of selectedNodeIds.current) {
            const connections = connectionLookup.get(nodeId);
            if (!connections)
                continue;
            for (const { edgeId } of connections.values()) {
                const edge = edgeLookup.get(edgeId);
                if (edge && (edge.selectable ?? edgesSelectable)) {
                    selectedEdgeIds.current.add(edgeId);
                }
            }
        }
        if (!areSetsEqual(prevSelectedNodeIds, selectedNodeIds.current)) {
            const changes = getSelectionChanges(nodeLookup, selectedNodeIds.current, true);
            triggerNodeChanges(changes);
        }
        if (!areSetsEqual(prevSelectedEdgeIds, selectedEdgeIds.current)) {
            const changes = getSelectionChanges(edgeLookup, selectedEdgeIds.current);
            triggerEdgeChanges(changes);
        }
        store.setState({
            userSelectionRect: nextUserSelectRect,
            userSelectionActive: true,
            nodesSelectionActive: false,
        });
    };
    const onPointerUp = (event) => {
        if (event.button !== 0 || !selectionStarted.current) {
            return;
        }
        event.target?.releasePointerCapture?.(event.pointerId);
        const { userSelectionRect } = store.getState();
        /*
         * We only want to trigger click functions when in selection mode if
         * the user did not move the mouse.
         */
        if (!userSelectionActive && userSelectionRect && event.target === container.current) {
            onClick?.(event);
        }
        store.setState({
            userSelectionActive: false,
            userSelectionRect: null,
            nodesSelectionActive: selectedNodeIds.current.size > 0,
        });
        onSelectionEnd?.(event);
        /*
         * If the user kept holding the selectionKey during the selection,
         * we need to reset the selectionInProgress, so the next click event is not prevented
         */
        if (selectionKeyPressed || selectionOnDrag) {
            selectionInProgress.current = false;
        }
        selectionStarted.current = false;
    };
    const draggable = panOnDrag === true || (Array.isArray(panOnDrag) && panOnDrag.includes(0));
    return (jsxs("div", { className: cc(['react-flow__pane', { draggable, dragging, selection: isSelecting }]), onClick: hasActiveSelection ? undefined : wrapHandler(onClick, container), onContextMenu: wrapHandler(onContextMenu, container), onWheel: wrapHandler(onWheel, container), onPointerEnter: hasActiveSelection ? undefined : onPaneMouseEnter, onPointerDown: hasActiveSelection ? onPointerDown : onPaneMouseMove, onPointerMove: hasActiveSelection ? onPointerMove : onPaneMouseMove, onPointerUp: hasActiveSelection ? onPointerUp : undefined, onPointerLeave: onPaneMouseLeave, ref: container, style: containerStyle, children: [children, jsx(UserSelection, {})] }));
}

/*
 * this handler is called by
 * 1. the click handler when node is not draggable or selectNodesOnDrag = false
 * or
 * 2. the on drag start handler when node is draggable and selectNodesOnDrag = true
 */
function handleNodeClick({ id, store, unselect = false, nodeRef, }) {
    const { addSelectedNodes, unselectNodesAndEdges, multiSelectionActive, nodeLookup, onError } = store.getState();
    const node = nodeLookup.get(id);
    if (!node) {
        onError?.('012', errorMessages['error012'](id));
        return;
    }
    store.setState({ nodesSelectionActive: false });
    if (!node.selected) {
        addSelectedNodes([id]);
    }
    else if (unselect || (node.selected && multiSelectionActive)) {
        unselectNodesAndEdges({ nodes: [node], edges: [] });
        requestAnimationFrame(() => nodeRef?.current?.blur());
    }
}

/**
 * Hook for calling XYDrag helper from @xyflow/system.
 *
 * @internal
 */
function useDrag({ nodeRef, disabled = false, noDragClassName, handleSelector, nodeId, isSelectable, nodeClickDistance, }) {
    const store = useStoreApi();
    const [dragging, setDragging] = useState(false);
    const xyDrag = useRef();
    useEffect(() => {
        xyDrag.current = XYDrag({
            getStoreItems: () => store.getState(),
            onNodeMouseDown: (id) => {
                handleNodeClick({
                    id,
                    store,
                    nodeRef,
                });
            },
            onDragStart: () => {
                setDragging(true);
            },
            onDragStop: () => {
                setDragging(false);
            },
        });
    }, []);
    useEffect(() => {
        if (disabled) {
            xyDrag.current?.destroy();
        }
        else if (nodeRef.current) {
            xyDrag.current?.update({
                noDragClassName,
                handleSelector,
                domNode: nodeRef.current,
                isSelectable,
                nodeId,
                nodeClickDistance,
            });
            return () => {
                xyDrag.current?.destroy();
            };
        }
    }, [noDragClassName, handleSelector, disabled, isSelectable, nodeRef, nodeId]);
    return dragging;
}

const selectedAndDraggable = (nodesDraggable) => (n) => n.selected && (n.draggable || (nodesDraggable && typeof n.draggable === 'undefined'));
/**
 * Hook for updating node positions by passing a direction and factor
 *
 * @internal
 * @returns function for updating node positions
 */
function useMoveSelectedNodes() {
    const store = useStoreApi();
    const moveSelectedNodes = useCallback((params) => {
        const { nodeExtent, snapToGrid, snapGrid, nodesDraggable, onError, updateNodePositions, nodeLookup, nodeOrigin } = store.getState();
        const nodeUpdates = new Map();
        const isSelected = selectedAndDraggable(nodesDraggable);
        /*
         * by default a node moves 5px on each key press
         * if snap grid is enabled, we use that for the velocity
         */
        const xVelo = snapToGrid ? snapGrid[0] : 5;
        const yVelo = snapToGrid ? snapGrid[1] : 5;
        const xDiff = params.direction.x * xVelo * params.factor;
        const yDiff = params.direction.y * yVelo * params.factor;
        for (const [, node] of nodeLookup) {
            if (!isSelected(node)) {
                continue;
            }
            let nextPosition = {
                x: node.internals.positionAbsolute.x + xDiff,
                y: node.internals.positionAbsolute.y + yDiff,
            };
            if (snapToGrid) {
                nextPosition = snapPosition(nextPosition, snapGrid);
            }
            const { position, positionAbsolute } = calculateNodePosition({
                nodeId: node.id,
                nextPosition,
                nodeLookup,
                nodeExtent,
                nodeOrigin,
                onError,
            });
            node.position = position;
            node.internals.positionAbsolute = positionAbsolute;
            nodeUpdates.set(node.id, node);
        }
        updateNodePositions(nodeUpdates);
    }, []);
    return moveSelectedNodes;
}

const NodeIdContext = createContext(null);
const Provider = NodeIdContext.Provider;
NodeIdContext.Consumer;
/**
 * You can use this hook to get the id of the node it is used inside. It is useful
 * if you need the node's id deeper in the render tree but don't want to manually
 * drill down the id as a prop.
 *
 * @public
 * @returns The id for a node in the flow.
 *
 * @example
 *```jsx
 *import { useNodeId } from '@xyflow/react';
 *
 *export default function CustomNode() {
 *  return (
 *    <div>
 *      <span>This node has an id of </span>
 *      <NodeIdDisplay />
 *    </div>
 *  );
 *}
 *
 *function NodeIdDisplay() {
 *  const nodeId = useNodeId();
 *
 *  return <span>{nodeId}</span>;
 *}
 *```
 */
const useNodeId = () => {
    const nodeId = useContext(NodeIdContext);
    return nodeId;
};

const selector$g = (s) => ({
    connectOnClick: s.connectOnClick,
    noPanClassName: s.noPanClassName,
    rfId: s.rfId,
});
const connectingSelector = (nodeId, handleId, type) => (state) => {
    const { connectionClickStartHandle: clickHandle, connectionMode, connection } = state;
    const { fromHandle, toHandle, isValid } = connection;
    const connectingTo = toHandle?.nodeId === nodeId && toHandle?.id === handleId && toHandle?.type === type;
    return {
        connectingFrom: fromHandle?.nodeId === nodeId && fromHandle?.id === handleId && fromHandle?.type === type,
        connectingTo,
        clickConnecting: clickHandle?.nodeId === nodeId && clickHandle?.id === handleId && clickHandle?.type === type,
        isPossibleEndHandle: connectionMode === ConnectionMode.Strict
            ? fromHandle?.type !== type
            : nodeId !== fromHandle?.nodeId || handleId !== fromHandle?.id,
        connectionInProcess: !!fromHandle,
        clickConnectionInProcess: !!clickHandle,
        valid: connectingTo && isValid,
    };
};
function HandleComponent({ type = 'source', position = Position.Top, isValidConnection, isConnectable = true, isConnectableStart = true, isConnectableEnd = true, id, onConnect, children, className, onMouseDown, onTouchStart, ...rest }, ref) {
    const handleId = id || null;
    const isTarget = type === 'target';
    const store = useStoreApi();
    const nodeId = useNodeId();
    const { connectOnClick, noPanClassName, rfId } = useStore(selector$g, shallow);
    const { connectingFrom, connectingTo, clickConnecting, isPossibleEndHandle, connectionInProcess, clickConnectionInProcess, valid, } = useStore(connectingSelector(nodeId, handleId, type), shallow);
    if (!nodeId) {
        store.getState().onError?.('010', errorMessages['error010']());
    }
    const onConnectExtended = (params) => {
        const { defaultEdgeOptions, onConnect: onConnectAction, hasDefaultEdges } = store.getState();
        const edgeParams = {
            ...defaultEdgeOptions,
            ...params,
        };
        if (hasDefaultEdges) {
            const { edges, setEdges } = store.getState();
            setEdges(addEdge(edgeParams, edges));
        }
        onConnectAction?.(edgeParams);
        onConnect?.(edgeParams);
    };
    const onPointerDown = (event) => {
        if (!nodeId) {
            return;
        }
        const isMouseTriggered = isMouseEvent(event.nativeEvent);
        if (isConnectableStart &&
            ((isMouseTriggered && event.button === 0) || !isMouseTriggered)) {
            const currentStore = store.getState();
            XYHandle.onPointerDown(event.nativeEvent, {
                autoPanOnConnect: currentStore.autoPanOnConnect,
                connectionMode: currentStore.connectionMode,
                connectionRadius: currentStore.connectionRadius,
                domNode: currentStore.domNode,
                nodeLookup: currentStore.nodeLookup,
                lib: currentStore.lib,
                isTarget,
                handleId,
                nodeId,
                flowId: currentStore.rfId,
                panBy: currentStore.panBy,
                cancelConnection: currentStore.cancelConnection,
                onConnectStart: currentStore.onConnectStart,
                onConnectEnd: currentStore.onConnectEnd,
                updateConnection: currentStore.updateConnection,
                onConnect: onConnectExtended,
                isValidConnection: isValidConnection || currentStore.isValidConnection,
                getTransform: () => store.getState().transform,
                getFromHandle: () => store.getState().connection.fromHandle,
                autoPanSpeed: currentStore.autoPanSpeed,
                dragThreshold: currentStore.connectionDragThreshold,
            });
        }
        if (isMouseTriggered) {
            onMouseDown?.(event);
        }
        else {
            onTouchStart?.(event);
        }
    };
    const onClick = (event) => {
        const { onClickConnectStart, onClickConnectEnd, connectionClickStartHandle, connectionMode, isValidConnection: isValidConnectionStore, lib, rfId: flowId, nodeLookup, connection: connectionState, } = store.getState();
        if (!nodeId || (!connectionClickStartHandle && !isConnectableStart)) {
            return;
        }
        if (!connectionClickStartHandle) {
            onClickConnectStart?.(event.nativeEvent, { nodeId, handleId, handleType: type });
            store.setState({ connectionClickStartHandle: { nodeId, type, id: handleId } });
            return;
        }
        const doc = getHostForElement(event.target);
        const isValidConnectionHandler = isValidConnection || isValidConnectionStore;
        const { connection, isValid } = XYHandle.isValid(event.nativeEvent, {
            handle: {
                nodeId,
                id: handleId,
                type,
            },
            connectionMode,
            fromNodeId: connectionClickStartHandle.nodeId,
            fromHandleId: connectionClickStartHandle.id || null,
            fromType: connectionClickStartHandle.type,
            isValidConnection: isValidConnectionHandler,
            flowId,
            doc,
            lib,
            nodeLookup,
        });
        if (isValid && connection) {
            onConnectExtended(connection);
        }
        const connectionClone = structuredClone(connectionState);
        delete connectionClone.inProgress;
        connectionClone.toPosition = connectionClone.toHandle ? connectionClone.toHandle.position : null;
        onClickConnectEnd?.(event, connectionClone);
        store.setState({ connectionClickStartHandle: null });
    };
    return (jsx("div", { "data-handleid": handleId, "data-nodeid": nodeId, "data-handlepos": position, "data-id": `${rfId}-${nodeId}-${handleId}-${type}`, className: cc([
            'react-flow__handle',
            `react-flow__handle-${position}`,
            'nodrag',
            noPanClassName,
            className,
            {
                source: !isTarget,
                target: isTarget,
                connectable: isConnectable,
                connectablestart: isConnectableStart,
                connectableend: isConnectableEnd,
                clickconnecting: clickConnecting,
                connectingfrom: connectingFrom,
                connectingto: connectingTo,
                valid,
                /*
                 * shows where you can start a connection from
                 * and where you can end it while connecting
                 */
                connectionindicator: isConnectable &&
                    (!connectionInProcess || isPossibleEndHandle) &&
                    (connectionInProcess || clickConnectionInProcess ? isConnectableEnd : isConnectableStart),
            },
        ]), onMouseDown: onPointerDown, onTouchStart: onPointerDown, onClick: connectOnClick ? onClick : undefined, ref: ref, ...rest, children: children }));
}
/**
 * The `<Handle />` component is used in your [custom nodes](/learn/customization/custom-nodes)
 * to define connection points.
 *
 *@public
 *
 *@example
 *
 *```jsx
 *import { Handle, Position } from '@xyflow/react';
 *
 *export function CustomNode({ data }) {
 *  return (
 *    <>
 *      <div style={{ padding: '10px 20px' }}>
 *        {data.label}
 *      </div>
 *
 *      <Handle type="target" position={Position.Left} />
 *      <Handle type="source" position={Position.Right} />
 *    </>
 *  );
 *};
 *```
 */
const Handle = memo(fixedForwardRef(HandleComponent));

function InputNode({ data, isConnectable, sourcePosition = Position.Bottom }) {
    return (jsxs(Fragment, { children: [data?.label, jsx(Handle, { type: "source", position: sourcePosition, isConnectable: isConnectable })] }));
}

function DefaultNode({ data, isConnectable, targetPosition = Position.Top, sourcePosition = Position.Bottom, }) {
    return (jsxs(Fragment, { children: [jsx(Handle, { type: "target", position: targetPosition, isConnectable: isConnectable }), data?.label, jsx(Handle, { type: "source", position: sourcePosition, isConnectable: isConnectable })] }));
}

function GroupNode() {
    return null;
}

function OutputNode({ data, isConnectable, targetPosition = Position.Top }) {
    return (jsxs(Fragment, { children: [jsx(Handle, { type: "target", position: targetPosition, isConnectable: isConnectable }), data?.label] }));
}

const arrowKeyDiffs = {
    ArrowUp: { x: 0, y: -1 },
    ArrowDown: { x: 0, y: 1 },
    ArrowLeft: { x: -1, y: 0 },
    ArrowRight: { x: 1, y: 0 },
};
const builtinNodeTypes = {
    input: InputNode,
    default: DefaultNode,
    output: OutputNode,
    group: GroupNode,
};
function getNodeInlineStyleDimensions(node) {
    if (node.internals.handleBounds === undefined) {
        return {
            width: node.width ?? node.initialWidth ?? node.style?.width,
            height: node.height ?? node.initialHeight ?? node.style?.height,
        };
    }
    return {
        width: node.width ?? node.style?.width,
        height: node.height ?? node.style?.height,
    };
}

const selector$f = (s) => {
    const { width, height, x, y } = getInternalNodesBounds(s.nodeLookup, {
        filter: (node) => !!node.selected,
    });
    return {
        width: isNumeric(width) ? width : null,
        height: isNumeric(height) ? height : null,
        userSelectionActive: s.userSelectionActive,
        transformString: `translate(${s.transform[0]}px,${s.transform[1]}px) scale(${s.transform[2]}) translate(${x}px,${y}px)`,
    };
};
function NodesSelection({ onSelectionContextMenu, noPanClassName, disableKeyboardA11y, }) {
    const store = useStoreApi();
    const { width, height, transformString, userSelectionActive } = useStore(selector$f, shallow);
    const moveSelectedNodes = useMoveSelectedNodes();
    const nodeRef = useRef(null);
    useEffect(() => {
        if (!disableKeyboardA11y) {
            nodeRef.current?.focus({
                preventScroll: true,
            });
        }
    }, [disableKeyboardA11y]);
    useDrag({
        nodeRef,
    });
    if (userSelectionActive || !width || !height) {
        return null;
    }
    const onContextMenu = onSelectionContextMenu
        ? (event) => {
            const selectedNodes = store.getState().nodes.filter((n) => n.selected);
            onSelectionContextMenu(event, selectedNodes);
        }
        : undefined;
    const onKeyDown = (event) => {
        if (Object.prototype.hasOwnProperty.call(arrowKeyDiffs, event.key)) {
            event.preventDefault();
            moveSelectedNodes({
                direction: arrowKeyDiffs[event.key],
                factor: event.shiftKey ? 4 : 1,
            });
        }
    };
    return (jsx("div", { className: cc(['react-flow__nodesselection', 'react-flow__container', noPanClassName]), style: {
            transform: transformString,
        }, children: jsx("div", { ref: nodeRef, className: "react-flow__nodesselection-rect", onContextMenu: onContextMenu, tabIndex: disableKeyboardA11y ? undefined : -1, onKeyDown: disableKeyboardA11y ? undefined : onKeyDown, style: {
                width,
                height,
            } }) }));
}

const win = undefined;
const selector$e = (s) => {
    return { nodesSelectionActive: s.nodesSelectionActive, userSelectionActive: s.userSelectionActive };
};
function FlowRendererComponent({ children, onPaneClick, onPaneMouseEnter, onPaneMouseMove, onPaneMouseLeave, onPaneContextMenu, onPaneScroll, paneClickDistance, deleteKeyCode, selectionKeyCode, selectionOnDrag, selectionMode, onSelectionStart, onSelectionEnd, multiSelectionKeyCode, panActivationKeyCode, zoomActivationKeyCode, elementsSelectable, zoomOnScroll, zoomOnPinch, panOnScroll: _panOnScroll, panOnScrollSpeed, panOnScrollMode, zoomOnDoubleClick, panOnDrag: _panOnDrag, defaultViewport, translateExtent, minZoom, maxZoom, preventScrolling, onSelectionContextMenu, noWheelClassName, noPanClassName, disableKeyboardA11y, onViewportChange, isControlledViewport, }) {
    const { nodesSelectionActive, userSelectionActive } = useStore(selector$e);
    const selectionKeyPressed = useKeyPress(selectionKeyCode, { target: win });
    const panActivationKeyPressed = useKeyPress(panActivationKeyCode, { target: win });
    const panOnDrag = panActivationKeyPressed || _panOnDrag;
    const panOnScroll = panActivationKeyPressed || _panOnScroll;
    const _selectionOnDrag = selectionOnDrag && panOnDrag !== true;
    const isSelecting = selectionKeyPressed || userSelectionActive || _selectionOnDrag;
    useGlobalKeyHandler({ deleteKeyCode, multiSelectionKeyCode });
    return (jsx(ZoomPane, { onPaneContextMenu: onPaneContextMenu, elementsSelectable: elementsSelectable, zoomOnScroll: zoomOnScroll, zoomOnPinch: zoomOnPinch, panOnScroll: panOnScroll, panOnScrollSpeed: panOnScrollSpeed, panOnScrollMode: panOnScrollMode, zoomOnDoubleClick: zoomOnDoubleClick, panOnDrag: !selectionKeyPressed && panOnDrag, defaultViewport: defaultViewport, translateExtent: translateExtent, minZoom: minZoom, maxZoom: maxZoom, zoomActivationKeyCode: zoomActivationKeyCode, preventScrolling: preventScrolling, noWheelClassName: noWheelClassName, noPanClassName: noPanClassName, onViewportChange: onViewportChange, isControlledViewport: isControlledViewport, paneClickDistance: paneClickDistance, children: jsxs(Pane, { onSelectionStart: onSelectionStart, onSelectionEnd: onSelectionEnd, onPaneClick: onPaneClick, onPaneMouseEnter: onPaneMouseEnter, onPaneMouseMove: onPaneMouseMove, onPaneMouseLeave: onPaneMouseLeave, onPaneContextMenu: onPaneContextMenu, onPaneScroll: onPaneScroll, panOnDrag: panOnDrag, isSelecting: !!isSelecting, selectionMode: selectionMode, selectionKeyPressed: selectionKeyPressed, selectionOnDrag: _selectionOnDrag, children: [children, nodesSelectionActive && (jsx(NodesSelection, { onSelectionContextMenu: onSelectionContextMenu, noPanClassName: noPanClassName, disableKeyboardA11y: disableKeyboardA11y }))] }) }));
}
FlowRendererComponent.displayName = 'FlowRenderer';
const FlowRenderer = memo(FlowRendererComponent);

const selector$d = (onlyRenderVisible) => (s) => {
    return onlyRenderVisible
        ? getNodesInside(s.nodeLookup, { x: 0, y: 0, width: s.width, height: s.height }, s.transform, true).map((node) => node.id)
        : Array.from(s.nodeLookup.keys());
};
/**
 * Hook for getting the visible node ids from the store.
 *
 * @internal
 * @param onlyRenderVisible
 * @returns array with visible node ids
 */
function useVisibleNodeIds(onlyRenderVisible) {
    const nodeIds = useStore(useCallback(selector$d(onlyRenderVisible), [onlyRenderVisible]), shallow);
    return nodeIds;
}

const selector$c = (s) => s.updateNodeInternals;
function useResizeObserver() {
    const updateNodeInternals = useStore(selector$c);
    const [resizeObserver] = useState(() => {
        if (typeof ResizeObserver === 'undefined') {
            return null;
        }
        return new ResizeObserver((entries) => {
            const updates = new Map();
            entries.forEach((entry) => {
                const id = entry.target.getAttribute('data-id');
                updates.set(id, {
                    id,
                    nodeElement: entry.target,
                    force: true,
                });
            });
            updateNodeInternals(updates);
        });
    });
    useEffect(() => {
        return () => {
            resizeObserver?.disconnect();
        };
    }, [resizeObserver]);
    return resizeObserver;
}

/**
 * Hook to handle the resize observation + internal updates for the passed node.
 *
 * @internal
 * @returns nodeRef - reference to the node element
 */
function useNodeObserver({ node, nodeType, hasDimensions, resizeObserver, }) {
    const store = useStoreApi();
    const nodeRef = useRef(null);
    const observedNode = useRef(null);
    const prevSourcePosition = useRef(node.sourcePosition);
    const prevTargetPosition = useRef(node.targetPosition);
    const prevType = useRef(nodeType);
    const isInitialized = hasDimensions && !!node.internals.handleBounds;
    useEffect(() => {
        if (nodeRef.current && !node.hidden && (!isInitialized || observedNode.current !== nodeRef.current)) {
            if (observedNode.current) {
                resizeObserver?.unobserve(observedNode.current);
            }
            resizeObserver?.observe(nodeRef.current);
            observedNode.current = nodeRef.current;
        }
    }, [isInitialized, node.hidden]);
    useEffect(() => {
        return () => {
            if (observedNode.current) {
                resizeObserver?.unobserve(observedNode.current);
                observedNode.current = null;
            }
        };
    }, []);
    useEffect(() => {
        if (nodeRef.current) {
            /*
             * when the user programmatically changes the source or handle position, we need to update the internals
             * to make sure the edges are updated correctly
             */
            const typeChanged = prevType.current !== nodeType;
            const sourcePosChanged = prevSourcePosition.current !== node.sourcePosition;
            const targetPosChanged = prevTargetPosition.current !== node.targetPosition;
            if (typeChanged || sourcePosChanged || targetPosChanged) {
                prevType.current = nodeType;
                prevSourcePosition.current = node.sourcePosition;
                prevTargetPosition.current = node.targetPosition;
                store
                    .getState()
                    .updateNodeInternals(new Map([[node.id, { id: node.id, nodeElement: nodeRef.current, force: true }]]));
            }
        }
    }, [node.id, nodeType, node.sourcePosition, node.targetPosition]);
    return nodeRef;
}

function NodeWrapper({ id, onClick, onMouseEnter, onMouseMove, onMouseLeave, onContextMenu, onDoubleClick, nodesDraggable, elementsSelectable, nodesConnectable, nodesFocusable, resizeObserver, noDragClassName, noPanClassName, disableKeyboardA11y, rfId, nodeTypes, nodeClickDistance, onError, }) {
    const { node, internals, isParent } = useStore((s) => {
        const node = s.nodeLookup.get(id);
        const isParent = s.parentLookup.has(id);
        return {
            node,
            internals: node.internals,
            isParent,
        };
    }, shallow);
    let nodeType = node.type || 'default';
    let NodeComponent = nodeTypes?.[nodeType] || builtinNodeTypes[nodeType];
    if (NodeComponent === undefined) {
        onError?.('003', errorMessages['error003'](nodeType));
        nodeType = 'default';
        NodeComponent = nodeTypes?.['default'] || builtinNodeTypes.default;
    }
    const isDraggable = !!(node.draggable || (nodesDraggable && typeof node.draggable === 'undefined'));
    const isSelectable = !!(node.selectable || (elementsSelectable && typeof node.selectable === 'undefined'));
    const isConnectable = !!(node.connectable || (nodesConnectable && typeof node.connectable === 'undefined'));
    const isFocusable = !!(node.focusable || (nodesFocusable && typeof node.focusable === 'undefined'));
    const store = useStoreApi();
    const hasDimensions = nodeHasDimensions(node);
    const nodeRef = useNodeObserver({ node, nodeType, hasDimensions, resizeObserver });
    const dragging = useDrag({
        nodeRef,
        disabled: node.hidden || !isDraggable,
        noDragClassName,
        handleSelector: node.dragHandle,
        nodeId: id,
        isSelectable,
        nodeClickDistance,
    });
    const moveSelectedNodes = useMoveSelectedNodes();
    if (node.hidden) {
        return null;
    }
    const nodeDimensions = getNodeDimensions(node);
    const inlineDimensions = getNodeInlineStyleDimensions(node);
    const hasPointerEvents = isSelectable || isDraggable || onClick || onMouseEnter || onMouseMove || onMouseLeave;
    const onMouseEnterHandler = onMouseEnter
        ? (event) => onMouseEnter(event, { ...internals.userNode })
        : undefined;
    const onMouseMoveHandler = onMouseMove
        ? (event) => onMouseMove(event, { ...internals.userNode })
        : undefined;
    const onMouseLeaveHandler = onMouseLeave
        ? (event) => onMouseLeave(event, { ...internals.userNode })
        : undefined;
    const onContextMenuHandler = onContextMenu
        ? (event) => onContextMenu(event, { ...internals.userNode })
        : undefined;
    const onDoubleClickHandler = onDoubleClick
        ? (event) => onDoubleClick(event, { ...internals.userNode })
        : undefined;
    const onSelectNodeHandler = (event) => {
        const { selectNodesOnDrag, nodeDragThreshold } = store.getState();
        if (isSelectable && (!selectNodesOnDrag || !isDraggable || nodeDragThreshold > 0)) {
            /*
             * this handler gets called by XYDrag on drag start when selectNodesOnDrag=true
             * here we only need to call it when selectNodesOnDrag=false
             */
            handleNodeClick({
                id,
                store,
                nodeRef,
            });
        }
        if (onClick) {
            onClick(event, { ...internals.userNode });
        }
    };
    const onKeyDown = (event) => {
        if (isInputDOMNode(event.nativeEvent) || disableKeyboardA11y) {
            return;
        }
        if (elementSelectionKeys.includes(event.key) && isSelectable) {
            const unselect = event.key === 'Escape';
            handleNodeClick({
                id,
                store,
                unselect,
                nodeRef,
            });
        }
        else if (isDraggable && node.selected && Object.prototype.hasOwnProperty.call(arrowKeyDiffs, event.key)) {
            // prevent default scrolling behavior on arrow key press when node is moved
            event.preventDefault();
            const { ariaLabelConfig } = store.getState();
            store.setState({
                ariaLiveMessage: ariaLabelConfig['node.a11yDescription.ariaLiveMessage']({
                    direction: event.key.replace('Arrow', '').toLowerCase(),
                    x: ~~internals.positionAbsolute.x,
                    y: ~~internals.positionAbsolute.y,
                }),
            });
            moveSelectedNodes({
                direction: arrowKeyDiffs[event.key],
                factor: event.shiftKey ? 4 : 1,
            });
        }
    };
    const onFocus = () => {
        if (disableKeyboardA11y || !nodeRef.current?.matches(':focus-visible')) {
            return;
        }
        const { transform, width, height, autoPanOnNodeFocus, setCenter } = store.getState();
        if (!autoPanOnNodeFocus) {
            return;
        }
        const withinViewport = getNodesInside(new Map([[id, node]]), { x: 0, y: 0, width, height }, transform, true).length > 0;
        if (!withinViewport) {
            setCenter(node.position.x + nodeDimensions.width / 2, node.position.y + nodeDimensions.height / 2, {
                zoom: transform[2],
            });
        }
    };
    return (jsx("div", { className: cc([
            'react-flow__node',
            `react-flow__node-${nodeType}`,
            {
                // this is overwritable by passing `nopan` as a class name
                [noPanClassName]: isDraggable,
            },
            node.className,
            {
                selected: node.selected,
                selectable: isSelectable,
                parent: isParent,
                draggable: isDraggable,
                dragging,
            },
        ]), ref: nodeRef, style: {
            zIndex: internals.z,
            transform: `translate(${internals.positionAbsolute.x}px,${internals.positionAbsolute.y}px)`,
            pointerEvents: hasPointerEvents ? 'all' : 'none',
            visibility: hasDimensions ? 'visible' : 'hidden',
            ...node.style,
            ...inlineDimensions,
        }, "data-id": id, "data-testid": `rf__node-${id}`, onMouseEnter: onMouseEnterHandler, onMouseMove: onMouseMoveHandler, onMouseLeave: onMouseLeaveHandler, onContextMenu: onContextMenuHandler, onClick: onSelectNodeHandler, onDoubleClick: onDoubleClickHandler, onKeyDown: isFocusable ? onKeyDown : undefined, tabIndex: isFocusable ? 0 : undefined, onFocus: isFocusable ? onFocus : undefined, role: node.ariaRole ?? (isFocusable ? 'group' : undefined), "aria-roledescription": "node", "aria-describedby": disableKeyboardA11y ? undefined : `${ARIA_NODE_DESC_KEY}-${rfId}`, "aria-label": node.ariaLabel, ...node.domAttributes, children: jsx(Provider, { value: id, children: jsx(NodeComponent, { id: id, data: node.data, type: nodeType, positionAbsoluteX: internals.positionAbsolute.x, positionAbsoluteY: internals.positionAbsolute.y, selected: node.selected ?? false, selectable: isSelectable, draggable: isDraggable, deletable: node.deletable ?? true, isConnectable: isConnectable, sourcePosition: node.sourcePosition, targetPosition: node.targetPosition, dragging: dragging, dragHandle: node.dragHandle, zIndex: internals.z, parentId: node.parentId, ...nodeDimensions }) }) }));
}

const selector$b = (s) => ({
    nodesDraggable: s.nodesDraggable,
    nodesConnectable: s.nodesConnectable,
    nodesFocusable: s.nodesFocusable,
    elementsSelectable: s.elementsSelectable,
    onError: s.onError,
});
function NodeRendererComponent(props) {
    const { nodesDraggable, nodesConnectable, nodesFocusable, elementsSelectable, onError } = useStore(selector$b, shallow);
    const nodeIds = useVisibleNodeIds(props.onlyRenderVisibleElements);
    const resizeObserver = useResizeObserver();
    return (jsx("div", { className: "react-flow__nodes", style: containerStyle, children: nodeIds.map((nodeId) => {
            return (
            /*
             * The split of responsibilities between NodeRenderer and
             * NodeComponentWrapper may appear weird. However, it’s designed to
             * minimize the cost of updates when individual nodes change.
             *
             * For example, when you’re dragging a single node, that node gets
             * updated multiple times per second. If `NodeRenderer` were to update
             * every time, it would have to re-run the `nodes.map()` loop every
             * time. This gets pricey with hundreds of nodes, especially if every
             * loop cycle does more than just rendering a JSX element!
             *
             * As a result of this choice, we took the following implementation
             * decisions:
             * - NodeRenderer subscribes *only* to node IDs – and therefore
             *   rerender *only* when visible nodes are added or removed.
             * - NodeRenderer performs all operations the result of which can be
             *   shared between nodes (such as creating the `ResizeObserver`
             *   instance, or subscribing to `selector`). This means extra prop
             *   drilling into `NodeComponentWrapper`, but it means we need to run
             *   these operations only once – instead of once per node.
             * - Any operations that you’d normally write inside `nodes.map` are
             *   moved into `NodeComponentWrapper`. This ensures they are
             *   memorized – so if `NodeRenderer` *has* to rerender, it only
             *   needs to regenerate the list of nodes, nothing else.
             */
            jsx(NodeWrapper, { id: nodeId, nodeTypes: props.nodeTypes, nodeExtent: props.nodeExtent, onClick: props.onNodeClick, onMouseEnter: props.onNodeMouseEnter, onMouseMove: props.onNodeMouseMove, onMouseLeave: props.onNodeMouseLeave, onContextMenu: props.onNodeContextMenu, onDoubleClick: props.onNodeDoubleClick, noDragClassName: props.noDragClassName, noPanClassName: props.noPanClassName, rfId: props.rfId, disableKeyboardA11y: props.disableKeyboardA11y, resizeObserver: resizeObserver, nodesDraggable: nodesDraggable, nodesConnectable: nodesConnectable, nodesFocusable: nodesFocusable, elementsSelectable: elementsSelectable, nodeClickDistance: props.nodeClickDistance, onError: onError }, nodeId));
        }) }));
}
NodeRendererComponent.displayName = 'NodeRenderer';
const NodeRenderer = memo(NodeRendererComponent);

/**
 * Hook for getting the visible edge ids from the store.
 *
 * @internal
 * @param onlyRenderVisible
 * @returns array with visible edge ids
 */
function useVisibleEdgeIds(onlyRenderVisible) {
    const edgeIds = useStore(useCallback((s) => {
        if (!onlyRenderVisible) {
            return s.edges.map((edge) => edge.id);
        }
        const visibleEdgeIds = [];
        if (s.width && s.height) {
            for (const edge of s.edges) {
                const sourceNode = s.nodeLookup.get(edge.source);
                const targetNode = s.nodeLookup.get(edge.target);
                if (sourceNode &&
                    targetNode &&
                    isEdgeVisible({
                        sourceNode,
                        targetNode,
                        width: s.width,
                        height: s.height,
                        transform: s.transform,
                    })) {
                    visibleEdgeIds.push(edge.id);
                }
            }
        }
        return visibleEdgeIds;
    }, [onlyRenderVisible]), shallow);
    return edgeIds;
}

const ArrowSymbol = ({ color = 'none', strokeWidth = 1 }) => {
    return (jsx("polyline", { style: {
            stroke: color,
            strokeWidth,
        }, strokeLinecap: "round", strokeLinejoin: "round", fill: "none", points: "-5,-4 0,0 -5,4" }));
};
const ArrowClosedSymbol = ({ color = 'none', strokeWidth = 1 }) => {
    return (jsx("polyline", { style: {
            stroke: color,
            fill: color,
            strokeWidth,
        }, strokeLinecap: "round", strokeLinejoin: "round", points: "-5,-4 0,0 -5,4 -5,-4" }));
};
const MarkerSymbols = {
    [MarkerType.Arrow]: ArrowSymbol,
    [MarkerType.ArrowClosed]: ArrowClosedSymbol,
};
function useMarkerSymbol(type) {
    const store = useStoreApi();
    const symbol = useMemo(() => {
        const symbolExists = Object.prototype.hasOwnProperty.call(MarkerSymbols, type);
        if (!symbolExists) {
            store.getState().onError?.('009', errorMessages['error009'](type));
            return null;
        }
        return MarkerSymbols[type];
    }, [type]);
    return symbol;
}

const Marker = ({ id, type, color, width = 12.5, height = 12.5, markerUnits = 'strokeWidth', strokeWidth, orient = 'auto-start-reverse', }) => {
    const Symbol = useMarkerSymbol(type);
    if (!Symbol) {
        return null;
    }
    return (jsx("marker", { className: "react-flow__arrowhead", id: id, markerWidth: `${width}`, markerHeight: `${height}`, viewBox: "-10 -10 20 20", markerUnits: markerUnits, orient: orient, refX: "0", refY: "0", children: jsx(Symbol, { color: color, strokeWidth: strokeWidth }) }));
};
/*
 * when you have multiple flows on a page and you hide the first one, the other ones have no markers anymore
 * when they do have markers with the same ids. To prevent this the user can pass a unique id to the react flow wrapper
 * that we can then use for creating our unique marker ids
 */
const MarkerDefinitions = ({ defaultColor, rfId }) => {
    const edges = useStore((s) => s.edges);
    const defaultEdgeOptions = useStore((s) => s.defaultEdgeOptions);
    const markers = useMemo(() => {
        const markers = createMarkerIds(edges, {
            id: rfId,
            defaultColor,
            defaultMarkerStart: defaultEdgeOptions?.markerStart,
            defaultMarkerEnd: defaultEdgeOptions?.markerEnd,
        });
        return markers;
    }, [edges, defaultEdgeOptions, rfId, defaultColor]);
    if (!markers.length) {
        return null;
    }
    return (jsx("svg", { className: "react-flow__marker", "aria-hidden": "true", children: jsx("defs", { children: markers.map((marker) => (jsx(Marker, { id: marker.id, type: marker.type, color: marker.color, width: marker.width, height: marker.height, markerUnits: marker.markerUnits, strokeWidth: marker.strokeWidth, orient: marker.orient }, marker.id))) }) }));
};
MarkerDefinitions.displayName = 'MarkerDefinitions';
var MarkerDefinitions$1 = memo(MarkerDefinitions);

function EdgeTextComponent({ x, y, label, labelStyle, labelShowBg = true, labelBgStyle, labelBgPadding = [2, 4], labelBgBorderRadius = 2, children, className, ...rest }) {
    const [edgeTextBbox, setEdgeTextBbox] = useState({ x: 1, y: 0, width: 0, height: 0 });
    const edgeTextClasses = cc(['react-flow__edge-textwrapper', className]);
    const edgeTextRef = useRef(null);
    useEffect(() => {
        if (edgeTextRef.current) {
            const textBbox = edgeTextRef.current.getBBox();
            setEdgeTextBbox({
                x: textBbox.x,
                y: textBbox.y,
                width: textBbox.width,
                height: textBbox.height,
            });
        }
    }, [label]);
    if (!label) {
        return null;
    }
    return (jsxs("g", { transform: `translate(${x - edgeTextBbox.width / 2} ${y - edgeTextBbox.height / 2})`, className: edgeTextClasses, visibility: edgeTextBbox.width ? 'visible' : 'hidden', ...rest, children: [labelShowBg && (jsx("rect", { width: edgeTextBbox.width + 2 * labelBgPadding[0], x: -labelBgPadding[0], y: -labelBgPadding[1], height: edgeTextBbox.height + 2 * labelBgPadding[1], className: "react-flow__edge-textbg", style: labelBgStyle, rx: labelBgBorderRadius, ry: labelBgBorderRadius })), jsx("text", { className: "react-flow__edge-text", y: edgeTextBbox.height / 2, dy: "0.3em", ref: edgeTextRef, style: labelStyle, children: label }), children] }));
}
EdgeTextComponent.displayName = 'EdgeText';
/**
 * You can use the `<EdgeText />` component as a helper component to display text
 * within your custom edges.
 *
 * @public
 *
 * @example
 * ```jsx
 * import { EdgeText } from '@xyflow/react';
 *
 * export function CustomEdgeLabel({ label }) {
 *   return (
 *     <EdgeText
 *       x={100}
 *       y={100}
 *       label={label}
 *       labelStyle={{ fill: 'white' }}
 *       labelShowBg
 *       labelBgStyle={{ fill: 'red' }}
 *       labelBgPadding={[2, 4]}
 *       labelBgBorderRadius={2}
 *     />
 *   );
 * }
 *```
 */
const EdgeText = memo(EdgeTextComponent);

/**
 * The `<BaseEdge />` component gets used internally for all the edges. It can be
 * used inside a custom edge and handles the invisible helper edge and the edge label
 * for you.
 *
 * @public
 * @example
 * ```jsx
 *import { BaseEdge } from '@xyflow/react';
 *
 *export function CustomEdge({ sourceX, sourceY, targetX, targetY, ...props }) {
 *  const [edgePath] = getStraightPath({
 *    sourceX,
 *    sourceY,
 *    targetX,
 *    targetY,
 *  });
 *
 *  return <BaseEdge path={edgePath} {...props} />;
 *}
 *```
 *
 * @remarks If you want to use an edge marker with the [`<BaseEdge />`](/api-reference/components/base-edge) component,
 * you can pass the `markerStart` or `markerEnd` props passed to your custom edge
 * through to the [`<BaseEdge />`](/api-reference/components/base-edge) component.
 * You can see all the props passed to a custom edge by looking at the [`EdgeProps`](/api-reference/types/edge-props) type.
 */
function BaseEdge({ path, labelX, labelY, label, labelStyle, labelShowBg, labelBgStyle, labelBgPadding, labelBgBorderRadius, interactionWidth = 20, ...props }) {
    return (jsxs(Fragment, { children: [jsx("path", { ...props, d: path, fill: "none", className: cc(['react-flow__edge-path', props.className]) }), interactionWidth && (jsx("path", { d: path, fill: "none", strokeOpacity: 0, strokeWidth: interactionWidth, className: "react-flow__edge-interaction" })), label && isNumeric(labelX) && isNumeric(labelY) ? (jsx(EdgeText, { x: labelX, y: labelY, label: label, labelStyle: labelStyle, labelShowBg: labelShowBg, labelBgStyle: labelBgStyle, labelBgPadding: labelBgPadding, labelBgBorderRadius: labelBgBorderRadius })) : null] }));
}

function getControl({ pos, x1, y1, x2, y2 }) {
    if (pos === Position.Left || pos === Position.Right) {
        return [0.5 * (x1 + x2), y1];
    }
    return [x1, 0.5 * (y1 + y2)];
}
/**
 * The `getSimpleBezierPath` util returns everything you need to render a simple
 * bezier edge between two nodes.
 * @public
 * @returns
 * - `path`: the path to use in an SVG `<path>` element.
 * - `labelX`: the `x` position you can use to render a label for this edge.
 * - `labelY`: the `y` position you can use to render a label for this edge.
 * - `offsetX`: the absolute difference between the source `x` position and the `x` position of the
 * middle of this path.
 * - `offsetY`: the absolute difference between the source `y` position and the `y` position of the
 * middle of this path.
 */
function getSimpleBezierPath({ sourceX, sourceY, sourcePosition = Position.Bottom, targetX, targetY, targetPosition = Position.Top, }) {
    const [sourceControlX, sourceControlY] = getControl({
        pos: sourcePosition,
        x1: sourceX,
        y1: sourceY,
        x2: targetX,
        y2: targetY,
    });
    const [targetControlX, targetControlY] = getControl({
        pos: targetPosition,
        x1: targetX,
        y1: targetY,
        x2: sourceX,
        y2: sourceY,
    });
    const [labelX, labelY, offsetX, offsetY] = getBezierEdgeCenter({
        sourceX,
        sourceY,
        targetX,
        targetY,
        sourceControlX,
        sourceControlY,
        targetControlX,
        targetControlY,
    });
    return [
        `M${sourceX},${sourceY} C${sourceControlX},${sourceControlY} ${targetControlX},${targetControlY} ${targetX},${targetY}`,
        labelX,
        labelY,
        offsetX,
        offsetY,
    ];
}
function createSimpleBezierEdge(params) {
    // eslint-disable-next-line react/display-name
    return memo(({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, label, labelStyle, labelShowBg, labelBgStyle, labelBgPadding, labelBgBorderRadius, style, markerEnd, markerStart, interactionWidth, }) => {
        const [path, labelX, labelY] = getSimpleBezierPath({
            sourceX,
            sourceY,
            sourcePosition,
            targetX,
            targetY,
            targetPosition,
        });
        const _id = params.isInternal ? undefined : id;
        return (jsx(BaseEdge, { id: _id, path: path, labelX: labelX, labelY: labelY, label: label, labelStyle: labelStyle, labelShowBg: labelShowBg, labelBgStyle: labelBgStyle, labelBgPadding: labelBgPadding, labelBgBorderRadius: labelBgBorderRadius, style: style, markerEnd: markerEnd, markerStart: markerStart, interactionWidth: interactionWidth }));
    });
}
const SimpleBezierEdge = createSimpleBezierEdge({ isInternal: false });
const SimpleBezierEdgeInternal = createSimpleBezierEdge({ isInternal: true });
SimpleBezierEdge.displayName = 'SimpleBezierEdge';
SimpleBezierEdgeInternal.displayName = 'SimpleBezierEdgeInternal';

function createSmoothStepEdge(params) {
    // eslint-disable-next-line react/display-name
    return memo(({ id, sourceX, sourceY, targetX, targetY, label, labelStyle, labelShowBg, labelBgStyle, labelBgPadding, labelBgBorderRadius, style, sourcePosition = Position.Bottom, targetPosition = Position.Top, markerEnd, markerStart, pathOptions, interactionWidth, }) => {
        const [path, labelX, labelY] = getSmoothStepPath({
            sourceX,
            sourceY,
            sourcePosition,
            targetX,
            targetY,
            targetPosition,
            borderRadius: pathOptions?.borderRadius,
            offset: pathOptions?.offset,
            stepPosition: pathOptions?.stepPosition,
        });
        const _id = params.isInternal ? undefined : id;
        return (jsx(BaseEdge, { id: _id, path: path, labelX: labelX, labelY: labelY, label: label, labelStyle: labelStyle, labelShowBg: labelShowBg, labelBgStyle: labelBgStyle, labelBgPadding: labelBgPadding, labelBgBorderRadius: labelBgBorderRadius, style: style, markerEnd: markerEnd, markerStart: markerStart, interactionWidth: interactionWidth }));
    });
}
/**
 * Component that can be used inside a custom edge to render a smooth step edge.
 *
 * @public
 * @example
 *
 * ```tsx
 * import { SmoothStepEdge } from '@xyflow/react';
 *
 * function CustomEdge({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition }) {
 *   return (
 *     <SmoothStepEdge
 *       sourceX={sourceX}
 *       sourceY={sourceY}
 *       targetX={targetX}
 *       targetY={targetY}
 *       sourcePosition={sourcePosition}
 *       targetPosition={targetPosition}
 *     />
 *   );
 * }
 * ```
 */
const SmoothStepEdge = createSmoothStepEdge({ isInternal: false });
/**
 * @internal
 */
const SmoothStepEdgeInternal = createSmoothStepEdge({ isInternal: true });
SmoothStepEdge.displayName = 'SmoothStepEdge';
SmoothStepEdgeInternal.displayName = 'SmoothStepEdgeInternal';

function createStepEdge(params) {
    // eslint-disable-next-line react/display-name
    return memo(({ id, ...props }) => {
        const _id = params.isInternal ? undefined : id;
        return (jsx(SmoothStepEdge, { ...props, id: _id, pathOptions: useMemo(() => ({ borderRadius: 0, offset: props.pathOptions?.offset }), [props.pathOptions?.offset]) }));
    });
}
/**
 * Component that can be used inside a custom edge to render a step edge.
 *
 * @public
 * @example
 *
 * ```tsx
 * import { StepEdge } from '@xyflow/react';
 *
 * function CustomEdge({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition }) {
 *   return (
 *     <StepEdge
 *       sourceX={sourceX}
 *       sourceY={sourceY}
 *       targetX={targetX}
 *       targetY={targetY}
 *       sourcePosition={sourcePosition}
 *       targetPosition={targetPosition}
 *     />
 *   );
 * }
 * ```
 */
const StepEdge = createStepEdge({ isInternal: false });
/**
 * @internal
 */
const StepEdgeInternal = createStepEdge({ isInternal: true });
StepEdge.displayName = 'StepEdge';
StepEdgeInternal.displayName = 'StepEdgeInternal';

function createStraightEdge(params) {
    // eslint-disable-next-line react/display-name
    return memo(({ id, sourceX, sourceY, targetX, targetY, label, labelStyle, labelShowBg, labelBgStyle, labelBgPadding, labelBgBorderRadius, style, markerEnd, markerStart, interactionWidth, }) => {
        const [path, labelX, labelY] = getStraightPath({ sourceX, sourceY, targetX, targetY });
        const _id = params.isInternal ? undefined : id;
        return (jsx(BaseEdge, { id: _id, path: path, labelX: labelX, labelY: labelY, label: label, labelStyle: labelStyle, labelShowBg: labelShowBg, labelBgStyle: labelBgStyle, labelBgPadding: labelBgPadding, labelBgBorderRadius: labelBgBorderRadius, style: style, markerEnd: markerEnd, markerStart: markerStart, interactionWidth: interactionWidth }));
    });
}
/**
 * Component that can be used inside a custom edge to render a straight line.
 *
 * @public
 * @example
 *
 * ```tsx
 * import { StraightEdge } from '@xyflow/react';
 *
 * function CustomEdge({ sourceX, sourceY, targetX, targetY }) {
 *   return (
 *     <StraightEdge
 *       sourceX={sourceX}
 *       sourceY={sourceY}
 *       targetX={targetX}
 *       targetY={targetY}
 *     />
 *   );
 * }
 * ```
 */
const StraightEdge = createStraightEdge({ isInternal: false });
/**
 * @internal
 */
const StraightEdgeInternal = createStraightEdge({ isInternal: true });
StraightEdge.displayName = 'StraightEdge';
StraightEdgeInternal.displayName = 'StraightEdgeInternal';

function createBezierEdge(params) {
    // eslint-disable-next-line react/display-name
    return memo(({ id, sourceX, sourceY, targetX, targetY, sourcePosition = Position.Bottom, targetPosition = Position.Top, label, labelStyle, labelShowBg, labelBgStyle, labelBgPadding, labelBgBorderRadius, style, markerEnd, markerStart, pathOptions, interactionWidth, }) => {
        const [path, labelX, labelY] = getBezierPath({
            sourceX,
            sourceY,
            sourcePosition,
            targetX,
            targetY,
            targetPosition,
            curvature: pathOptions?.curvature,
        });
        const _id = params.isInternal ? undefined : id;
        return (jsx(BaseEdge, { id: _id, path: path, labelX: labelX, labelY: labelY, label: label, labelStyle: labelStyle, labelShowBg: labelShowBg, labelBgStyle: labelBgStyle, labelBgPadding: labelBgPadding, labelBgBorderRadius: labelBgBorderRadius, style: style, markerEnd: markerEnd, markerStart: markerStart, interactionWidth: interactionWidth }));
    });
}
/**
 * Component that can be used inside a custom edge to render a bezier curve.
 *
 * @public
 * @example
 *
 * ```tsx
 * import { BezierEdge } from '@xyflow/react';
 *
 * function CustomEdge({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition }) {
 *   return (
 *     <BezierEdge
 *       sourceX={sourceX}
 *       sourceY={sourceY}
 *       targetX={targetX}
 *       targetY={targetY}
 *       sourcePosition={sourcePosition}
 *       targetPosition={targetPosition}
 *     />
 *   );
 * }
 * ```
 */
const BezierEdge = createBezierEdge({ isInternal: false });
/**
 * @internal
 */
const BezierEdgeInternal = createBezierEdge({ isInternal: true });
BezierEdge.displayName = 'BezierEdge';
BezierEdgeInternal.displayName = 'BezierEdgeInternal';

const builtinEdgeTypes = {
    default: BezierEdgeInternal,
    straight: StraightEdgeInternal,
    step: StepEdgeInternal,
    smoothstep: SmoothStepEdgeInternal,
    simplebezier: SimpleBezierEdgeInternal,
};
const nullPosition = {
    sourceX: null,
    sourceY: null,
    targetX: null,
    targetY: null,
    sourcePosition: null,
    targetPosition: null,
};

const shiftX = (x, shift, position) => {
    if (position === Position.Left)
        return x - shift;
    if (position === Position.Right)
        return x + shift;
    return x;
};
const shiftY = (y, shift, position) => {
    if (position === Position.Top)
        return y - shift;
    if (position === Position.Bottom)
        return y + shift;
    return y;
};
const EdgeUpdaterClassName = 'react-flow__edgeupdater';
/**
 * @internal
 */
function EdgeAnchor({ position, centerX, centerY, radius = 10, onMouseDown, onMouseEnter, onMouseOut, type, }) {
    return (jsx("circle", { onMouseDown: onMouseDown, onMouseEnter: onMouseEnter, onMouseOut: onMouseOut, className: cc([EdgeUpdaterClassName, `${EdgeUpdaterClassName}-${type}`]), cx: shiftX(centerX, radius, position), cy: shiftY(centerY, radius, position), r: radius, stroke: "transparent", fill: "transparent" }));
}

function EdgeUpdateAnchors({ isReconnectable, reconnectRadius, edge, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, onReconnect, onReconnectStart, onReconnectEnd, setReconnecting, setUpdateHover, }) {
    const store = useStoreApi();
    const handleEdgeUpdater = (event, oppositeHandle) => {
        // avoid triggering edge updater if mouse btn is not left
        if (event.button !== 0) {
            return;
        }
        const { autoPanOnConnect, domNode, isValidConnection, connectionMode, connectionRadius, lib, onConnectStart, onConnectEnd, cancelConnection, nodeLookup, rfId: flowId, panBy, updateConnection, } = store.getState();
        const isTarget = oppositeHandle.type === 'target';
        const _onReconnectEnd = (evt, connectionState) => {
            setReconnecting(false);
            onReconnectEnd?.(evt, edge, oppositeHandle.type, connectionState);
        };
        const onConnectEdge = (connection) => onReconnect?.(edge, connection);
        const _onConnectStart = (_event, params) => {
            setReconnecting(true);
            onReconnectStart?.(event, edge, oppositeHandle.type);
            onConnectStart?.(_event, params);
        };
        XYHandle.onPointerDown(event.nativeEvent, {
            autoPanOnConnect,
            connectionMode,
            connectionRadius,
            domNode,
            handleId: oppositeHandle.id,
            nodeId: oppositeHandle.nodeId,
            nodeLookup,
            isTarget,
            edgeUpdaterType: oppositeHandle.type,
            lib,
            flowId,
            cancelConnection,
            panBy,
            isValidConnection,
            onConnect: onConnectEdge,
            onConnectStart: _onConnectStart,
            onConnectEnd,
            onReconnectEnd: _onReconnectEnd,
            updateConnection,
            getTransform: () => store.getState().transform,
            getFromHandle: () => store.getState().connection.fromHandle,
            dragThreshold: store.getState().connectionDragThreshold,
        });
    };
    const onReconnectSourceMouseDown = (event) => handleEdgeUpdater(event, { nodeId: edge.target, id: edge.targetHandle ?? null, type: 'target' });
    const onReconnectTargetMouseDown = (event) => handleEdgeUpdater(event, { nodeId: edge.source, id: edge.sourceHandle ?? null, type: 'source' });
    const onReconnectMouseEnter = () => setUpdateHover(true);
    const onReconnectMouseOut = () => setUpdateHover(false);
    return (jsxs(Fragment, { children: [(isReconnectable === true || isReconnectable === 'source') && (jsx(EdgeAnchor, { position: sourcePosition, centerX: sourceX, centerY: sourceY, radius: reconnectRadius, onMouseDown: onReconnectSourceMouseDown, onMouseEnter: onReconnectMouseEnter, onMouseOut: onReconnectMouseOut, type: "source" })), (isReconnectable === true || isReconnectable === 'target') && (jsx(EdgeAnchor, { position: targetPosition, centerX: targetX, centerY: targetY, radius: reconnectRadius, onMouseDown: onReconnectTargetMouseDown, onMouseEnter: onReconnectMouseEnter, onMouseOut: onReconnectMouseOut, type: "target" }))] }));
}

function EdgeWrapper({ id, edgesFocusable, edgesReconnectable, elementsSelectable, onClick, onDoubleClick, onContextMenu, onMouseEnter, onMouseMove, onMouseLeave, reconnectRadius, onReconnect, onReconnectStart, onReconnectEnd, rfId, edgeTypes, noPanClassName, onError, disableKeyboardA11y, }) {
    let edge = useStore((s) => s.edgeLookup.get(id));
    const defaultEdgeOptions = useStore((s) => s.defaultEdgeOptions);
    edge = defaultEdgeOptions ? { ...defaultEdgeOptions, ...edge } : edge;
    let edgeType = edge.type || 'default';
    let EdgeComponent = edgeTypes?.[edgeType] || builtinEdgeTypes[edgeType];
    if (EdgeComponent === undefined) {
        onError?.('011', errorMessages['error011'](edgeType));
        edgeType = 'default';
        EdgeComponent = edgeTypes?.['default'] || builtinEdgeTypes.default;
    }
    const isFocusable = !!(edge.focusable || (edgesFocusable && typeof edge.focusable === 'undefined'));
    const isReconnectable = typeof onReconnect !== 'undefined' &&
        (edge.reconnectable || (edgesReconnectable && typeof edge.reconnectable === 'undefined'));
    const isSelectable = !!(edge.selectable || (elementsSelectable && typeof edge.selectable === 'undefined'));
    const edgeRef = useRef(null);
    const [updateHover, setUpdateHover] = useState(false);
    const [reconnecting, setReconnecting] = useState(false);
    const store = useStoreApi();
    const { zIndex, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition } = useStore(useCallback((store) => {
        const sourceNode = store.nodeLookup.get(edge.source);
        const targetNode = store.nodeLookup.get(edge.target);
        if (!sourceNode || !targetNode) {
            return {
                zIndex: edge.zIndex,
                ...nullPosition,
            };
        }
        const edgePosition = getEdgePosition({
            id,
            sourceNode,
            targetNode,
            sourceHandle: edge.sourceHandle || null,
            targetHandle: edge.targetHandle || null,
            connectionMode: store.connectionMode,
            onError,
        });
        const zIndex = getElevatedEdgeZIndex({
            selected: edge.selected,
            zIndex: edge.zIndex,
            sourceNode,
            targetNode,
            elevateOnSelect: store.elevateEdgesOnSelect,
        });
        return {
            zIndex,
            ...(edgePosition || nullPosition),
        };
    }, [edge.source, edge.target, edge.sourceHandle, edge.targetHandle, edge.selected, edge.zIndex]), shallow);
    const markerStartUrl = useMemo(() => (edge.markerStart ? `url('#${getMarkerId(edge.markerStart, rfId)}')` : undefined), [edge.markerStart, rfId]);
    const markerEndUrl = useMemo(() => (edge.markerEnd ? `url('#${getMarkerId(edge.markerEnd, rfId)}')` : undefined), [edge.markerEnd, rfId]);
    if (edge.hidden || sourceX === null || sourceY === null || targetX === null || targetY === null) {
        return null;
    }
    const onEdgeClick = (event) => {
        const { addSelectedEdges, unselectNodesAndEdges, multiSelectionActive } = store.getState();
        if (isSelectable) {
            store.setState({ nodesSelectionActive: false });
            if (edge.selected && multiSelectionActive) {
                unselectNodesAndEdges({ nodes: [], edges: [edge] });
                edgeRef.current?.blur();
            }
            else {
                addSelectedEdges([id]);
            }
        }
        if (onClick) {
            onClick(event, edge);
        }
    };
    const onEdgeDoubleClick = onDoubleClick
        ? (event) => {
            onDoubleClick(event, { ...edge });
        }
        : undefined;
    const onEdgeContextMenu = onContextMenu
        ? (event) => {
            onContextMenu(event, { ...edge });
        }
        : undefined;
    const onEdgeMouseEnter = onMouseEnter
        ? (event) => {
            onMouseEnter(event, { ...edge });
        }
        : undefined;
    const onEdgeMouseMove = onMouseMove
        ? (event) => {
            onMouseMove(event, { ...edge });
        }
        : undefined;
    const onEdgeMouseLeave = onMouseLeave
        ? (event) => {
            onMouseLeave(event, { ...edge });
        }
        : undefined;
    const onKeyDown = (event) => {
        if (!disableKeyboardA11y && elementSelectionKeys.includes(event.key) && isSelectable) {
            const { unselectNodesAndEdges, addSelectedEdges } = store.getState();
            const unselect = event.key === 'Escape';
            if (unselect) {
                edgeRef.current?.blur();
                unselectNodesAndEdges({ edges: [edge] });
            }
            else {
                addSelectedEdges([id]);
            }
        }
    };
    return (jsx("svg", { style: { zIndex }, children: jsxs("g", { className: cc([
                'react-flow__edge',
                `react-flow__edge-${edgeType}`,
                edge.className,
                noPanClassName,
                {
                    selected: edge.selected,
                    animated: edge.animated,
                    inactive: !isSelectable && !onClick,
                    updating: updateHover,
                    selectable: isSelectable,
                },
            ]), onClick: onEdgeClick, onDoubleClick: onEdgeDoubleClick, onContextMenu: onEdgeContextMenu, onMouseEnter: onEdgeMouseEnter, onMouseMove: onEdgeMouseMove, onMouseLeave: onEdgeMouseLeave, onKeyDown: isFocusable ? onKeyDown : undefined, tabIndex: isFocusable ? 0 : undefined, role: edge.ariaRole ?? (isFocusable ? 'group' : 'img'), "aria-roledescription": "edge", "data-id": id, "data-testid": `rf__edge-${id}`, "aria-label": edge.ariaLabel === null ? undefined : edge.ariaLabel || `Edge from ${edge.source} to ${edge.target}`, "aria-describedby": isFocusable ? `${ARIA_EDGE_DESC_KEY}-${rfId}` : undefined, ref: edgeRef, ...edge.domAttributes, children: [!reconnecting && (jsx(EdgeComponent, { id: id, source: edge.source, target: edge.target, type: edge.type, selected: edge.selected, animated: edge.animated, selectable: isSelectable, deletable: edge.deletable ?? true, label: edge.label, labelStyle: edge.labelStyle, labelShowBg: edge.labelShowBg, labelBgStyle: edge.labelBgStyle, labelBgPadding: edge.labelBgPadding, labelBgBorderRadius: edge.labelBgBorderRadius, sourceX: sourceX, sourceY: sourceY, targetX: targetX, targetY: targetY, sourcePosition: sourcePosition, targetPosition: targetPosition, data: edge.data, style: edge.style, sourceHandleId: edge.sourceHandle, targetHandleId: edge.targetHandle, markerStart: markerStartUrl, markerEnd: markerEndUrl, pathOptions: 'pathOptions' in edge ? edge.pathOptions : undefined, interactionWidth: edge.interactionWidth })), isReconnectable && (jsx(EdgeUpdateAnchors, { edge: edge, isReconnectable: isReconnectable, reconnectRadius: reconnectRadius, onReconnect: onReconnect, onReconnectStart: onReconnectStart, onReconnectEnd: onReconnectEnd, sourceX: sourceX, sourceY: sourceY, targetX: targetX, targetY: targetY, sourcePosition: sourcePosition, targetPosition: targetPosition, setUpdateHover: setUpdateHover, setReconnecting: setReconnecting }))] }) }));
}

const selector$a = (s) => ({
    edgesFocusable: s.edgesFocusable,
    edgesReconnectable: s.edgesReconnectable,
    elementsSelectable: s.elementsSelectable,
    connectionMode: s.connectionMode,
    onError: s.onError,
});
function EdgeRendererComponent({ defaultMarkerColor, onlyRenderVisibleElements, rfId, edgeTypes, noPanClassName, onReconnect, onEdgeContextMenu, onEdgeMouseEnter, onEdgeMouseMove, onEdgeMouseLeave, onEdgeClick, reconnectRadius, onEdgeDoubleClick, onReconnectStart, onReconnectEnd, disableKeyboardA11y, }) {
    const { edgesFocusable, edgesReconnectable, elementsSelectable, onError } = useStore(selector$a, shallow);
    const edgeIds = useVisibleEdgeIds(onlyRenderVisibleElements);
    return (jsxs("div", { className: "react-flow__edges", children: [jsx(MarkerDefinitions$1, { defaultColor: defaultMarkerColor, rfId: rfId }), edgeIds.map((id) => {
                return (jsx(EdgeWrapper, { id: id, edgesFocusable: edgesFocusable, edgesReconnectable: edgesReconnectable, elementsSelectable: elementsSelectable, noPanClassName: noPanClassName, onReconnect: onReconnect, onContextMenu: onEdgeContextMenu, onMouseEnter: onEdgeMouseEnter, onMouseMove: onEdgeMouseMove, onMouseLeave: onEdgeMouseLeave, onClick: onEdgeClick, reconnectRadius: reconnectRadius, onDoubleClick: onEdgeDoubleClick, onReconnectStart: onReconnectStart, onReconnectEnd: onReconnectEnd, rfId: rfId, onError: onError, edgeTypes: edgeTypes, disableKeyboardA11y: disableKeyboardA11y }, id));
            })] }));
}
EdgeRendererComponent.displayName = 'EdgeRenderer';
const EdgeRenderer = memo(EdgeRendererComponent);

const selector$9 = (s) => `translate(${s.transform[0]}px,${s.transform[1]}px) scale(${s.transform[2]})`;
function Viewport({ children }) {
    const transform = useStore(selector$9);
    return (jsx("div", { className: "react-flow__viewport xyflow__viewport react-flow__container", style: { transform }, children: children }));
}

/**
 * Hook for calling onInit handler.
 *
 * @internal
 */
function useOnInitHandler(onInit) {
    const rfInstance = useReactFlow();
    const isInitialized = useRef(false);
    useEffect(() => {
        if (!isInitialized.current && rfInstance.viewportInitialized && onInit) {
            setTimeout(() => onInit(rfInstance), 1);
            isInitialized.current = true;
        }
    }, [onInit, rfInstance.viewportInitialized]);
}

const selector$8 = (state) => state.panZoom?.syncViewport;
/**
 * Hook for syncing the viewport with the panzoom instance.
 *
 * @internal
 * @param viewport
 */
function useViewportSync(viewport) {
    const syncViewport = useStore(selector$8);
    const store = useStoreApi();
    useEffect(() => {
        if (viewport) {
            syncViewport?.(viewport);
            store.setState({ transform: [viewport.x, viewport.y, viewport.zoom] });
        }
    }, [viewport, syncViewport]);
    return null;
}

function storeSelector$1(s) {
    return s.connection.inProgress
        ? { ...s.connection, to: pointToRendererPoint(s.connection.to, s.transform) }
        : { ...s.connection };
}
function getSelector(connectionSelector) {
    return storeSelector$1;
}
/**
 * The `useConnection` hook returns the current connection when there is an active
 * connection interaction. If no connection interaction is active, it returns null
 * for every property. A typical use case for this hook is to colorize handles
 * based on a certain condition (e.g. if the connection is valid or not).
 *
 * @public
 * @param connectionSelector - An optional selector function used to extract a slice of the
 * `ConnectionState` data. Using a selector can prevent component re-renders where data you don't
 * otherwise care about might change. If a selector is not provided, the entire `ConnectionState`
 * object is returned unchanged.
 * @example
 *
 * ```tsx
 *import { useConnection } from '@xyflow/react';
 *
 *function App() {
 *  const connection = useConnection();
 *
 *  return (
 *    <div> {connection ? `Someone is trying to make a connection from ${connection.fromNode} to this one.` : 'There are currently no incoming connections!'}
 *
 *   </div>
 *   );
 * }
 * ```
 *
 * @returns ConnectionState
 */
function useConnection(connectionSelector) {
    const combinedSelector = getSelector();
    return useStore(combinedSelector, shallow);
}

const selector$7 = (s) => ({
    nodesConnectable: s.nodesConnectable,
    isValid: s.connection.isValid,
    inProgress: s.connection.inProgress,
    width: s.width,
    height: s.height,
});
function ConnectionLineWrapper({ containerStyle, style, type, component, }) {
    const { nodesConnectable, width, height, isValid, inProgress } = useStore(selector$7, shallow);
    const renderConnection = !!(width && nodesConnectable && inProgress);
    if (!renderConnection) {
        return null;
    }
    return (jsx("svg", { style: containerStyle, width: width, height: height, className: "react-flow__connectionline react-flow__container", children: jsx("g", { className: cc(['react-flow__connection', getConnectionStatus(isValid)]), children: jsx(ConnectionLine, { style: style, type: type, CustomComponent: component, isValid: isValid }) }) }));
}
const ConnectionLine = ({ style, type = ConnectionLineType.Bezier, CustomComponent, isValid, }) => {
    const { inProgress, from, fromNode, fromHandle, fromPosition, to, toNode, toHandle, toPosition } = useConnection();
    if (!inProgress) {
        return;
    }
    if (CustomComponent) {
        return (jsx(CustomComponent, { connectionLineType: type, connectionLineStyle: style, fromNode: fromNode, fromHandle: fromHandle, fromX: from.x, fromY: from.y, toX: to.x, toY: to.y, fromPosition: fromPosition, toPosition: toPosition, connectionStatus: getConnectionStatus(isValid), toNode: toNode, toHandle: toHandle }));
    }
    let path = '';
    const pathParams = {
        sourceX: from.x,
        sourceY: from.y,
        sourcePosition: fromPosition,
        targetX: to.x,
        targetY: to.y,
        targetPosition: toPosition,
    };
    switch (type) {
        case ConnectionLineType.Bezier:
            [path] = getBezierPath(pathParams);
            break;
        case ConnectionLineType.SimpleBezier:
            [path] = getSimpleBezierPath(pathParams);
            break;
        case ConnectionLineType.Step:
            [path] = getSmoothStepPath({
                ...pathParams,
                borderRadius: 0,
            });
            break;
        case ConnectionLineType.SmoothStep:
            [path] = getSmoothStepPath(pathParams);
            break;
        default:
            [path] = getStraightPath(pathParams);
    }
    return jsx("path", { d: path, fill: "none", className: "react-flow__connection-path", style: style });
};
ConnectionLine.displayName = 'ConnectionLine';

const emptyTypes = {};
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function useNodeOrEdgeTypesWarning(nodeOrEdgeTypes = emptyTypes) {
    useRef(nodeOrEdgeTypes);
    useStoreApi();
    useEffect(() => {
    }, [nodeOrEdgeTypes]);
}

function useStylesLoadedWarning() {
    useStoreApi();
    useRef(false);
    useEffect(() => {
    }, []);
}

function GraphViewComponent({ nodeTypes, edgeTypes, onInit, onNodeClick, onEdgeClick, onNodeDoubleClick, onEdgeDoubleClick, onNodeMouseEnter, onNodeMouseMove, onNodeMouseLeave, onNodeContextMenu, onSelectionContextMenu, onSelectionStart, onSelectionEnd, connectionLineType, connectionLineStyle, connectionLineComponent, connectionLineContainerStyle, selectionKeyCode, selectionOnDrag, selectionMode, multiSelectionKeyCode, panActivationKeyCode, zoomActivationKeyCode, deleteKeyCode, onlyRenderVisibleElements, elementsSelectable, defaultViewport, translateExtent, minZoom, maxZoom, preventScrolling, defaultMarkerColor, zoomOnScroll, zoomOnPinch, panOnScroll, panOnScrollSpeed, panOnScrollMode, zoomOnDoubleClick, panOnDrag, onPaneClick, onPaneMouseEnter, onPaneMouseMove, onPaneMouseLeave, onPaneScroll, onPaneContextMenu, paneClickDistance, nodeClickDistance, onEdgeContextMenu, onEdgeMouseEnter, onEdgeMouseMove, onEdgeMouseLeave, reconnectRadius, onReconnect, onReconnectStart, onReconnectEnd, noDragClassName, noWheelClassName, noPanClassName, disableKeyboardA11y, nodeExtent, rfId, viewport, onViewportChange, }) {
    useNodeOrEdgeTypesWarning(nodeTypes);
    useNodeOrEdgeTypesWarning(edgeTypes);
    useStylesLoadedWarning();
    useOnInitHandler(onInit);
    useViewportSync(viewport);
    return (jsx(FlowRenderer, { onPaneClick: onPaneClick, onPaneMouseEnter: onPaneMouseEnter, onPaneMouseMove: onPaneMouseMove, onPaneMouseLeave: onPaneMouseLeave, onPaneContextMenu: onPaneContextMenu, onPaneScroll: onPaneScroll, paneClickDistance: paneClickDistance, deleteKeyCode: deleteKeyCode, selectionKeyCode: selectionKeyCode, selectionOnDrag: selectionOnDrag, selectionMode: selectionMode, onSelectionStart: onSelectionStart, onSelectionEnd: onSelectionEnd, multiSelectionKeyCode: multiSelectionKeyCode, panActivationKeyCode: panActivationKeyCode, zoomActivationKeyCode: zoomActivationKeyCode, elementsSelectable: elementsSelectable, zoomOnScroll: zoomOnScroll, zoomOnPinch: zoomOnPinch, zoomOnDoubleClick: zoomOnDoubleClick, panOnScroll: panOnScroll, panOnScrollSpeed: panOnScrollSpeed, panOnScrollMode: panOnScrollMode, panOnDrag: panOnDrag, defaultViewport: defaultViewport, translateExtent: translateExtent, minZoom: minZoom, maxZoom: maxZoom, onSelectionContextMenu: onSelectionContextMenu, preventScrolling: preventScrolling, noDragClassName: noDragClassName, noWheelClassName: noWheelClassName, noPanClassName: noPanClassName, disableKeyboardA11y: disableKeyboardA11y, onViewportChange: onViewportChange, isControlledViewport: !!viewport, children: jsxs(Viewport, { children: [jsx(EdgeRenderer, { edgeTypes: edgeTypes, onEdgeClick: onEdgeClick, onEdgeDoubleClick: onEdgeDoubleClick, onReconnect: onReconnect, onReconnectStart: onReconnectStart, onReconnectEnd: onReconnectEnd, onlyRenderVisibleElements: onlyRenderVisibleElements, onEdgeContextMenu: onEdgeContextMenu, onEdgeMouseEnter: onEdgeMouseEnter, onEdgeMouseMove: onEdgeMouseMove, onEdgeMouseLeave: onEdgeMouseLeave, reconnectRadius: reconnectRadius, defaultMarkerColor: defaultMarkerColor, noPanClassName: noPanClassName, disableKeyboardA11y: disableKeyboardA11y, rfId: rfId }), jsx(ConnectionLineWrapper, { style: connectionLineStyle, type: connectionLineType, component: connectionLineComponent, containerStyle: connectionLineContainerStyle }), jsx("div", { className: "react-flow__edgelabel-renderer" }), jsx(NodeRenderer, { nodeTypes: nodeTypes, onNodeClick: onNodeClick, onNodeDoubleClick: onNodeDoubleClick, onNodeMouseEnter: onNodeMouseEnter, onNodeMouseMove: onNodeMouseMove, onNodeMouseLeave: onNodeMouseLeave, onNodeContextMenu: onNodeContextMenu, nodeClickDistance: nodeClickDistance, onlyRenderVisibleElements: onlyRenderVisibleElements, noPanClassName: noPanClassName, noDragClassName: noDragClassName, disableKeyboardA11y: disableKeyboardA11y, nodeExtent: nodeExtent, rfId: rfId }), jsx("div", { className: "react-flow__viewport-portal" })] }) }));
}
GraphViewComponent.displayName = 'GraphView';
const GraphView = memo(GraphViewComponent);

const getInitialState = ({ nodes, edges, defaultNodes, defaultEdges, width, height, fitView, fitViewOptions, minZoom = 0.5, maxZoom = 2, nodeOrigin, nodeExtent, } = {}) => {
    const nodeLookup = new Map();
    const parentLookup = new Map();
    const connectionLookup = new Map();
    const edgeLookup = new Map();
    const storeEdges = defaultEdges ?? edges ?? [];
    const storeNodes = defaultNodes ?? nodes ?? [];
    const storeNodeOrigin = nodeOrigin ?? [0, 0];
    const storeNodeExtent = nodeExtent ?? infiniteExtent;
    updateConnectionLookup(connectionLookup, edgeLookup, storeEdges);
    const nodesInitialized = adoptUserNodes(storeNodes, nodeLookup, parentLookup, {
        nodeOrigin: storeNodeOrigin,
        nodeExtent: storeNodeExtent,
        elevateNodesOnSelect: false,
    });
    let transform = [0, 0, 1];
    if (fitView && width && height) {
        const bounds = getInternalNodesBounds(nodeLookup, {
            filter: (node) => !!((node.width || node.initialWidth) && (node.height || node.initialHeight)),
        });
        const { x, y, zoom } = getViewportForBounds(bounds, width, height, minZoom, maxZoom, fitViewOptions?.padding ?? 0.1);
        transform = [x, y, zoom];
    }
    return {
        rfId: '1',
        width: 0,
        height: 0,
        transform,
        nodes: storeNodes,
        nodesInitialized,
        nodeLookup,
        parentLookup,
        edges: storeEdges,
        edgeLookup,
        connectionLookup,
        onNodesChange: null,
        onEdgesChange: null,
        hasDefaultNodes: defaultNodes !== undefined,
        hasDefaultEdges: defaultEdges !== undefined,
        panZoom: null,
        minZoom,
        maxZoom,
        translateExtent: infiniteExtent,
        nodeExtent: storeNodeExtent,
        nodesSelectionActive: false,
        userSelectionActive: false,
        userSelectionRect: null,
        connectionMode: ConnectionMode.Strict,
        domNode: null,
        paneDragging: false,
        noPanClassName: 'nopan',
        nodeOrigin: storeNodeOrigin,
        nodeDragThreshold: 1,
        connectionDragThreshold: 1,
        snapGrid: [15, 15],
        snapToGrid: false,
        nodesDraggable: true,
        nodesConnectable: true,
        nodesFocusable: true,
        edgesFocusable: true,
        edgesReconnectable: true,
        elementsSelectable: true,
        elevateNodesOnSelect: true,
        elevateEdgesOnSelect: false,
        selectNodesOnDrag: true,
        multiSelectionActive: false,
        fitViewQueued: fitView ?? false,
        fitViewOptions,
        fitViewResolver: null,
        connection: { ...initialConnection },
        connectionClickStartHandle: null,
        connectOnClick: true,
        ariaLiveMessage: '',
        autoPanOnConnect: true,
        autoPanOnNodeDrag: true,
        autoPanOnNodeFocus: true,
        autoPanSpeed: 15,
        connectionRadius: 20,
        onError: devWarn,
        isValidConnection: undefined,
        onSelectionChangeHandlers: [],
        lib: 'react',
        debug: false,
        ariaLabelConfig: defaultAriaLabelConfig,
    };
};

const createStore = ({ nodes, edges, defaultNodes, defaultEdges, width, height, fitView, fitViewOptions, minZoom, maxZoom, nodeOrigin, nodeExtent, }) => createWithEqualityFn((set, get) => {
    async function resolveFitView() {
        const { nodeLookup, panZoom, fitViewOptions, fitViewResolver, width, height, minZoom, maxZoom } = get();
        if (!panZoom) {
            return;
        }
        await fitViewport({
            nodes: nodeLookup,
            width,
            height,
            panZoom,
            minZoom,
            maxZoom,
        }, fitViewOptions);
        fitViewResolver?.resolve(true);
        /**
         * wait for the fitViewport to resolve before deleting the resolver,
         * we want to reuse the old resolver if the user calls fitView again in the mean time
         */
        set({ fitViewResolver: null });
    }
    return {
        ...getInitialState({
            nodes,
            edges,
            width,
            height,
            fitView,
            fitViewOptions,
            minZoom,
            maxZoom,
            nodeOrigin,
            nodeExtent,
            defaultNodes,
            defaultEdges,
        }),
        setNodes: (nodes) => {
            const { nodeLookup, parentLookup, nodeOrigin, elevateNodesOnSelect, fitViewQueued } = get();
            /*
             * setNodes() is called exclusively in response to user actions:
             * - either when the `<ReactFlow nodes>` prop is updated in the controlled ReactFlow setup,
             * - or when the user calls something like `reactFlowInstance.setNodes()` in an uncontrolled ReactFlow setup.
             *
             * When this happens, we take the note objects passed by the user and extend them with fields
             * relevant for internal React Flow operations.
             */
            const nodesInitialized = adoptUserNodes(nodes, nodeLookup, parentLookup, {
                nodeOrigin,
                nodeExtent,
                elevateNodesOnSelect,
                checkEquality: true,
            });
            if (fitViewQueued && nodesInitialized) {
                resolveFitView();
                set({ nodes, nodesInitialized, fitViewQueued: false, fitViewOptions: undefined });
            }
            else {
                set({ nodes, nodesInitialized });
            }
        },
        setEdges: (edges) => {
            const { connectionLookup, edgeLookup } = get();
            updateConnectionLookup(connectionLookup, edgeLookup, edges);
            set({ edges });
        },
        setDefaultNodesAndEdges: (nodes, edges) => {
            if (nodes) {
                const { setNodes } = get();
                setNodes(nodes);
                set({ hasDefaultNodes: true });
            }
            if (edges) {
                const { setEdges } = get();
                setEdges(edges);
                set({ hasDefaultEdges: true });
            }
        },
        /*
         * Every node gets registerd at a ResizeObserver. Whenever a node
         * changes its dimensions, this function is called to measure the
         * new dimensions and update the nodes.
         */
        updateNodeInternals: (updates) => {
            const { triggerNodeChanges, nodeLookup, parentLookup, domNode, nodeOrigin, nodeExtent, debug, fitViewQueued } = get();
            const { changes, updatedInternals } = updateNodeInternals(updates, nodeLookup, parentLookup, domNode, nodeOrigin, nodeExtent);
            if (!updatedInternals) {
                return;
            }
            updateAbsolutePositions(nodeLookup, parentLookup, { nodeOrigin, nodeExtent });
            if (fitViewQueued) {
                resolveFitView();
                set({ fitViewQueued: false, fitViewOptions: undefined });
            }
            else {
                // we always want to trigger useStore calls whenever updateNodeInternals is called
                set({});
            }
            if (changes?.length > 0) {
                if (debug) {
                    console.log('React Flow: trigger node changes', changes);
                }
                triggerNodeChanges?.(changes);
            }
        },
        updateNodePositions: (nodeDragItems, dragging = false) => {
            const parentExpandChildren = [];
            const changes = [];
            const { nodeLookup, triggerNodeChanges } = get();
            for (const [id, dragItem] of nodeDragItems) {
                // we are using the nodelookup to be sure to use the current expandParent and parentId value
                const node = nodeLookup.get(id);
                const expandParent = !!(node?.expandParent && node?.parentId && dragItem?.position);
                const change = {
                    id,
                    type: 'position',
                    position: expandParent
                        ? {
                            x: Math.max(0, dragItem.position.x),
                            y: Math.max(0, dragItem.position.y),
                        }
                        : dragItem.position,
                    dragging,
                };
                if (expandParent && node.parentId) {
                    parentExpandChildren.push({
                        id,
                        parentId: node.parentId,
                        rect: {
                            ...dragItem.internals.positionAbsolute,
                            width: dragItem.measured.width ?? 0,
                            height: dragItem.measured.height ?? 0,
                        },
                    });
                }
                changes.push(change);
            }
            if (parentExpandChildren.length > 0) {
                const { parentLookup, nodeOrigin } = get();
                const parentExpandChanges = handleExpandParent(parentExpandChildren, nodeLookup, parentLookup, nodeOrigin);
                changes.push(...parentExpandChanges);
            }
            triggerNodeChanges(changes);
        },
        triggerNodeChanges: (changes) => {
            const { onNodesChange, setNodes, nodes, hasDefaultNodes, debug } = get();
            if (changes?.length) {
                if (hasDefaultNodes) {
                    const updatedNodes = applyNodeChanges(changes, nodes);
                    setNodes(updatedNodes);
                }
                if (debug) {
                    console.log('React Flow: trigger node changes', changes);
                }
                onNodesChange?.(changes);
            }
        },
        triggerEdgeChanges: (changes) => {
            const { onEdgesChange, setEdges, edges, hasDefaultEdges, debug } = get();
            if (changes?.length) {
                if (hasDefaultEdges) {
                    const updatedEdges = applyEdgeChanges(changes, edges);
                    setEdges(updatedEdges);
                }
                if (debug) {
                    console.log('React Flow: trigger edge changes', changes);
                }
                onEdgesChange?.(changes);
            }
        },
        addSelectedNodes: (selectedNodeIds) => {
            const { multiSelectionActive, edgeLookup, nodeLookup, triggerNodeChanges, triggerEdgeChanges } = get();
            if (multiSelectionActive) {
                const nodeChanges = selectedNodeIds.map((nodeId) => createSelectionChange(nodeId, true));
                triggerNodeChanges(nodeChanges);
                return;
            }
            triggerNodeChanges(getSelectionChanges(nodeLookup, new Set([...selectedNodeIds]), true));
            triggerEdgeChanges(getSelectionChanges(edgeLookup));
        },
        addSelectedEdges: (selectedEdgeIds) => {
            const { multiSelectionActive, edgeLookup, nodeLookup, triggerNodeChanges, triggerEdgeChanges } = get();
            if (multiSelectionActive) {
                const changedEdges = selectedEdgeIds.map((edgeId) => createSelectionChange(edgeId, true));
                triggerEdgeChanges(changedEdges);
                return;
            }
            triggerEdgeChanges(getSelectionChanges(edgeLookup, new Set([...selectedEdgeIds])));
            triggerNodeChanges(getSelectionChanges(nodeLookup, new Set(), true));
        },
        unselectNodesAndEdges: ({ nodes, edges } = {}) => {
            const { edges: storeEdges, nodes: storeNodes, nodeLookup, triggerNodeChanges, triggerEdgeChanges } = get();
            const nodesToUnselect = nodes ? nodes : storeNodes;
            const edgesToUnselect = edges ? edges : storeEdges;
            const nodeChanges = nodesToUnselect.map((n) => {
                const internalNode = nodeLookup.get(n.id);
                if (internalNode) {
                    /*
                     * we need to unselect the internal node that was selected previously before we
                     * send the change to the user to prevent it to be selected while dragging the new node
                     */
                    internalNode.selected = false;
                }
                return createSelectionChange(n.id, false);
            });
            const edgeChanges = edgesToUnselect.map((edge) => createSelectionChange(edge.id, false));
            triggerNodeChanges(nodeChanges);
            triggerEdgeChanges(edgeChanges);
        },
        setMinZoom: (minZoom) => {
            const { panZoom, maxZoom } = get();
            panZoom?.setScaleExtent([minZoom, maxZoom]);
            set({ minZoom });
        },
        setMaxZoom: (maxZoom) => {
            const { panZoom, minZoom } = get();
            panZoom?.setScaleExtent([minZoom, maxZoom]);
            set({ maxZoom });
        },
        setTranslateExtent: (translateExtent) => {
            get().panZoom?.setTranslateExtent(translateExtent);
            set({ translateExtent });
        },
        setPaneClickDistance: (clickDistance) => {
            get().panZoom?.setClickDistance(clickDistance);
        },
        resetSelectedElements: () => {
            const { edges, nodes, triggerNodeChanges, triggerEdgeChanges, elementsSelectable } = get();
            if (!elementsSelectable) {
                return;
            }
            const nodeChanges = nodes.reduce((res, node) => (node.selected ? [...res, createSelectionChange(node.id, false)] : res), []);
            const edgeChanges = edges.reduce((res, edge) => (edge.selected ? [...res, createSelectionChange(edge.id, false)] : res), []);
            triggerNodeChanges(nodeChanges);
            triggerEdgeChanges(edgeChanges);
        },
        setNodeExtent: (nextNodeExtent) => {
            const { nodes, nodeLookup, parentLookup, nodeOrigin, elevateNodesOnSelect, nodeExtent } = get();
            if (nextNodeExtent[0][0] === nodeExtent[0][0] &&
                nextNodeExtent[0][1] === nodeExtent[0][1] &&
                nextNodeExtent[1][0] === nodeExtent[1][0] &&
                nextNodeExtent[1][1] === nodeExtent[1][1]) {
                return;
            }
            adoptUserNodes(nodes, nodeLookup, parentLookup, {
                nodeOrigin,
                nodeExtent: nextNodeExtent,
                elevateNodesOnSelect,
                checkEquality: false,
            });
            set({ nodeExtent: nextNodeExtent });
        },
        panBy: (delta) => {
            const { transform, width, height, panZoom, translateExtent } = get();
            return panBy({ delta, panZoom, transform, translateExtent, width, height });
        },
        setCenter: async (x, y, options) => {
            const { width, height, maxZoom, panZoom } = get();
            if (!panZoom) {
                return Promise.resolve(false);
            }
            const nextZoom = typeof options?.zoom !== 'undefined' ? options.zoom : maxZoom;
            await panZoom.setViewport({
                x: width / 2 - x * nextZoom,
                y: height / 2 - y * nextZoom,
                zoom: nextZoom,
            }, { duration: options?.duration, ease: options?.ease, interpolate: options?.interpolate });
            return Promise.resolve(true);
        },
        cancelConnection: () => {
            set({
                connection: { ...initialConnection },
            });
        },
        updateConnection: (connection) => {
            set({ connection });
        },
        reset: () => set({ ...getInitialState() }),
    };
}, Object.is);

/**
 * The `<ReactFlowProvider />` component is a [context provider](https://react.dev/learn/passing-data-deeply-with-context#)
 * that makes it possible to access a flow's internal state outside of the
 * [`<ReactFlow />`](/api-reference/react-flow) component. Many of the hooks we
 * provide rely on this component to work.
 * @public
 *
 * @example
 * ```tsx
 *import { ReactFlow, ReactFlowProvider, useNodes } from '@xyflow/react'
 *
 *export default function Flow() {
 *  return (
 *    <ReactFlowProvider>
 *      <ReactFlow nodes={...} edges={...} />
 *      <Sidebar />
 *    </ReactFlowProvider>
 *  );
 *}
 *
 *function Sidebar() {
 *  // This hook will only work if the component it's used in is a child of a
 *  // <ReactFlowProvider />.
 *  const nodes = useNodes()
 *
 *  return <aside>do something with nodes</aside>;
 *}
 *```
 *
 * @remarks If you're using a router and want your flow's state to persist across routes,
 * it's vital that you place the `<ReactFlowProvider />` component _outside_ of
 * your router. If you have multiple flows on the same page you will need to use a separate
 * `<ReactFlowProvider />` for each flow.
 */
function ReactFlowProvider({ initialNodes: nodes, initialEdges: edges, defaultNodes, defaultEdges, initialWidth: width, initialHeight: height, initialMinZoom: minZoom, initialMaxZoom: maxZoom, initialFitViewOptions: fitViewOptions, fitView, nodeOrigin, nodeExtent, children, }) {
    const [store] = useState(() => createStore({
        nodes,
        edges,
        defaultNodes,
        defaultEdges,
        width,
        height,
        fitView,
        minZoom,
        maxZoom,
        fitViewOptions,
        nodeOrigin,
        nodeExtent,
    }));
    return (jsx(Provider$1, { value: store, children: jsx(BatchProvider, { children: children }) }));
}

function Wrapper({ children, nodes, edges, defaultNodes, defaultEdges, width, height, fitView, fitViewOptions, minZoom, maxZoom, nodeOrigin, nodeExtent, }) {
    const isWrapped = useContext(StoreContext);
    if (isWrapped) {
        /*
         * we need to wrap it with a fragment because it's not allowed for children to be a ReactNode
         * https://github.com/DefinitelyTyped/DefinitelyTyped/issues/18051
         */
        return jsx(Fragment, { children: children });
    }
    return (jsx(ReactFlowProvider, { initialNodes: nodes, initialEdges: edges, defaultNodes: defaultNodes, defaultEdges: defaultEdges, initialWidth: width, initialHeight: height, fitView: fitView, initialFitViewOptions: fitViewOptions, initialMinZoom: minZoom, initialMaxZoom: maxZoom, nodeOrigin: nodeOrigin, nodeExtent: nodeExtent, children: children }));
}

const wrapperStyle = {
    width: '100%',
    height: '100%',
    overflow: 'hidden',
    position: 'relative',
    zIndex: 0,
};
function ReactFlow({ nodes, edges, defaultNodes, defaultEdges, className, nodeTypes, edgeTypes, onNodeClick, onEdgeClick, onInit, onMove, onMoveStart, onMoveEnd, onConnect, onConnectStart, onConnectEnd, onClickConnectStart, onClickConnectEnd, onNodeMouseEnter, onNodeMouseMove, onNodeMouseLeave, onNodeContextMenu, onNodeDoubleClick, onNodeDragStart, onNodeDrag, onNodeDragStop, onNodesDelete, onEdgesDelete, onDelete, onSelectionChange, onSelectionDragStart, onSelectionDrag, onSelectionDragStop, onSelectionContextMenu, onSelectionStart, onSelectionEnd, onBeforeDelete, connectionMode, connectionLineType = ConnectionLineType.Bezier, connectionLineStyle, connectionLineComponent, connectionLineContainerStyle, deleteKeyCode = 'Backspace', selectionKeyCode = 'Shift', selectionOnDrag = false, selectionMode = SelectionMode.Full, panActivationKeyCode = 'Space', multiSelectionKeyCode = isMacOs() ? 'Meta' : 'Control', zoomActivationKeyCode = isMacOs() ? 'Meta' : 'Control', snapToGrid, snapGrid, onlyRenderVisibleElements = false, selectNodesOnDrag, nodesDraggable, autoPanOnNodeFocus, nodesConnectable, nodesFocusable, nodeOrigin = defaultNodeOrigin, edgesFocusable, edgesReconnectable, elementsSelectable = true, defaultViewport: defaultViewport$1 = defaultViewport, minZoom = 0.5, maxZoom = 2, translateExtent = infiniteExtent, preventScrolling = true, nodeExtent, defaultMarkerColor = '#b1b1b7', zoomOnScroll = true, zoomOnPinch = true, panOnScroll = false, panOnScrollSpeed = 0.5, panOnScrollMode = PanOnScrollMode.Free, zoomOnDoubleClick = true, panOnDrag = true, onPaneClick, onPaneMouseEnter, onPaneMouseMove, onPaneMouseLeave, onPaneScroll, onPaneContextMenu, paneClickDistance = 0, nodeClickDistance = 0, children, onReconnect, onReconnectStart, onReconnectEnd, onEdgeContextMenu, onEdgeDoubleClick, onEdgeMouseEnter, onEdgeMouseMove, onEdgeMouseLeave, reconnectRadius = 10, onNodesChange, onEdgesChange, noDragClassName = 'nodrag', noWheelClassName = 'nowheel', noPanClassName = 'nopan', fitView, fitViewOptions, connectOnClick, attributionPosition, proOptions, defaultEdgeOptions, elevateNodesOnSelect, elevateEdgesOnSelect, disableKeyboardA11y = false, autoPanOnConnect, autoPanOnNodeDrag, autoPanSpeed, connectionRadius, isValidConnection, onError, style, id, nodeDragThreshold, connectionDragThreshold, viewport, onViewportChange, width, height, colorMode = 'light', debug, onScroll, ariaLabelConfig, ...rest }, ref) {
    const rfId = id || '1';
    const colorModeClassName = useColorModeClass(colorMode);
    // Undo scroll events, preventing viewport from shifting when nodes outside of it are focused
    const wrapperOnScroll = useCallback((e) => {
        e.currentTarget.scrollTo({ top: 0, left: 0, behavior: 'instant' });
        onScroll?.(e);
    }, [onScroll]);
    return (jsx("div", { "data-testid": "rf__wrapper", ...rest, onScroll: wrapperOnScroll, style: { ...style, ...wrapperStyle }, ref: ref, className: cc(['react-flow', className, colorModeClassName]), id: id, role: "application", children: jsxs(Wrapper, { nodes: nodes, edges: edges, width: width, height: height, fitView: fitView, fitViewOptions: fitViewOptions, minZoom: minZoom, maxZoom: maxZoom, nodeOrigin: nodeOrigin, nodeExtent: nodeExtent, children: [jsx(GraphView, { onInit: onInit, onNodeClick: onNodeClick, onEdgeClick: onEdgeClick, onNodeMouseEnter: onNodeMouseEnter, onNodeMouseMove: onNodeMouseMove, onNodeMouseLeave: onNodeMouseLeave, onNodeContextMenu: onNodeContextMenu, onNodeDoubleClick: onNodeDoubleClick, nodeTypes: nodeTypes, edgeTypes: edgeTypes, connectionLineType: connectionLineType, connectionLineStyle: connectionLineStyle, connectionLineComponent: connectionLineComponent, connectionLineContainerStyle: connectionLineContainerStyle, selectionKeyCode: selectionKeyCode, selectionOnDrag: selectionOnDrag, selectionMode: selectionMode, deleteKeyCode: deleteKeyCode, multiSelectionKeyCode: multiSelectionKeyCode, panActivationKeyCode: panActivationKeyCode, zoomActivationKeyCode: zoomActivationKeyCode, onlyRenderVisibleElements: onlyRenderVisibleElements, defaultViewport: defaultViewport$1, translateExtent: translateExtent, minZoom: minZoom, maxZoom: maxZoom, preventScrolling: preventScrolling, zoomOnScroll: zoomOnScroll, zoomOnPinch: zoomOnPinch, zoomOnDoubleClick: zoomOnDoubleClick, panOnScroll: panOnScroll, panOnScrollSpeed: panOnScrollSpeed, panOnScrollMode: panOnScrollMode, panOnDrag: panOnDrag, onPaneClick: onPaneClick, onPaneMouseEnter: onPaneMouseEnter, onPaneMouseMove: onPaneMouseMove, onPaneMouseLeave: onPaneMouseLeave, onPaneScroll: onPaneScroll, onPaneContextMenu: onPaneContextMenu, paneClickDistance: paneClickDistance, nodeClickDistance: nodeClickDistance, onSelectionContextMenu: onSelectionContextMenu, onSelectionStart: onSelectionStart, onSelectionEnd: onSelectionEnd, onReconnect: onReconnect, onReconnectStart: onReconnectStart, onReconnectEnd: onReconnectEnd, onEdgeContextMenu: onEdgeContextMenu, onEdgeDoubleClick: onEdgeDoubleClick, onEdgeMouseEnter: onEdgeMouseEnter, onEdgeMouseMove: onEdgeMouseMove, onEdgeMouseLeave: onEdgeMouseLeave, reconnectRadius: reconnectRadius, defaultMarkerColor: defaultMarkerColor, noDragClassName: noDragClassName, noWheelClassName: noWheelClassName, noPanClassName: noPanClassName, rfId: rfId, disableKeyboardA11y: disableKeyboardA11y, nodeExtent: nodeExtent, viewport: viewport, onViewportChange: onViewportChange }), jsx(StoreUpdater, { nodes: nodes, edges: edges, defaultNodes: defaultNodes, defaultEdges: defaultEdges, onConnect: onConnect, onConnectStart: onConnectStart, onConnectEnd: onConnectEnd, onClickConnectStart: onClickConnectStart, onClickConnectEnd: onClickConnectEnd, nodesDraggable: nodesDraggable, autoPanOnNodeFocus: autoPanOnNodeFocus, nodesConnectable: nodesConnectable, nodesFocusable: nodesFocusable, edgesFocusable: edgesFocusable, edgesReconnectable: edgesReconnectable, elementsSelectable: elementsSelectable, elevateNodesOnSelect: elevateNodesOnSelect, elevateEdgesOnSelect: elevateEdgesOnSelect, minZoom: minZoom, maxZoom: maxZoom, nodeExtent: nodeExtent, onNodesChange: onNodesChange, onEdgesChange: onEdgesChange, snapToGrid: snapToGrid, snapGrid: snapGrid, connectionMode: connectionMode, translateExtent: translateExtent, connectOnClick: connectOnClick, defaultEdgeOptions: defaultEdgeOptions, fitView: fitView, fitViewOptions: fitViewOptions, onNodesDelete: onNodesDelete, onEdgesDelete: onEdgesDelete, onDelete: onDelete, onNodeDragStart: onNodeDragStart, onNodeDrag: onNodeDrag, onNodeDragStop: onNodeDragStop, onSelectionDrag: onSelectionDrag, onSelectionDragStart: onSelectionDragStart, onSelectionDragStop: onSelectionDragStop, onMove: onMove, onMoveStart: onMoveStart, onMoveEnd: onMoveEnd, noPanClassName: noPanClassName, nodeOrigin: nodeOrigin, rfId: rfId, autoPanOnConnect: autoPanOnConnect, autoPanOnNodeDrag: autoPanOnNodeDrag, autoPanSpeed: autoPanSpeed, onError: onError, connectionRadius: connectionRadius, isValidConnection: isValidConnection, selectNodesOnDrag: selectNodesOnDrag, nodeDragThreshold: nodeDragThreshold, connectionDragThreshold: connectionDragThreshold, onBeforeDelete: onBeforeDelete, paneClickDistance: paneClickDistance, debug: debug, ariaLabelConfig: ariaLabelConfig }), jsx(SelectionListener, { onSelectionChange: onSelectionChange }), children, jsx(Attribution, { proOptions: proOptions, position: attributionPosition }), jsx(A11yDescriptions, { rfId: rfId, disableKeyboardA11y: disableKeyboardA11y })] }) }));
}
/**
 * The `<ReactFlow />` component is the heart of your React Flow application.
 * It renders your nodes and edges and handles user interaction
 *
 * @public
 *
 * @example
 * ```tsx
 *import { ReactFlow } from '@xyflow/react'
 *
 *export default function Flow() {
 *  return (<ReactFlow
 *    nodes={...}
 *    edges={...}
 *    onNodesChange={...}
 *    ...
 *  />);
 *}
 *```
 */
var index = fixedForwardRef(ReactFlow);

/**
 * This hook makes it easy to prototype a controlled flow where you manage the
 * state of nodes and edges outside the `ReactFlowInstance`. You can think of it
 * like React's `useState` hook with an additional helper callback.
 *
 * @public
 * @returns
 * - `nodes`: The current array of nodes. You might pass this directly to the `nodes` prop of your
 * `<ReactFlow />` component, or you may want to manipulate it first to perform some layouting,
 * for example.
 * - `setNodes`: A function that you can use to update the nodes. You can pass it a new array of
 * nodes or a callback that receives the current array of nodes and returns a new array of nodes.
 * This is the same as the second element of the tuple returned by React's `useState` hook.
 * - `onNodesChange`: A handy callback that can take an array of `NodeChanges` and update the nodes
 * state accordingly. You'll typically pass this directly to the `onNodesChange` prop of your
 * `<ReactFlow />` component.
 * @example
 *
 *```tsx
 *import { ReactFlow, useNodesState, useEdgesState } from '@xyflow/react';
 *
 *const initialNodes = [];
 *const initialEdges = [];
 *
 *export default function () {
 *  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
 *  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
 *
 *  return (
 *    <ReactFlow
 *      nodes={nodes}
 *      edges={edges}
 *      onNodesChange={onNodesChange}
 *      onEdgesChange={onEdgesChange}
 *    />
 *  );
 *}
 *```
 *
 * @remarks This hook was created to make prototyping easier and our documentation
 * examples clearer. Although it is OK to use this hook in production, in
 * practice you may want to use a more sophisticated state management solution
 * like Zustand {@link https://reactflow.dev/docs/guides/state-management/} instead.
 *
 */
function useNodesState(initialNodes) {
    const [nodes, setNodes] = useState(initialNodes);
    const onNodesChange = useCallback((changes) => setNodes((nds) => applyNodeChanges(changes, nds)), []);
    return [nodes, setNodes, onNodesChange];
}
/**
 * This hook makes it easy to prototype a controlled flow where you manage the
 * state of nodes and edges outside the `ReactFlowInstance`. You can think of it
 * like React's `useState` hook with an additional helper callback.
 *
 * @public
 * @returns
 * - `edges`: The current array of edges. You might pass this directly to the `edges` prop of your
 * `<ReactFlow />` component, or you may want to manipulate it first to perform some layouting,
 * for example.
 *
 * - `setEdges`: A function that you can use to update the edges. You can pass it a new array of
 * edges or a callback that receives the current array of edges and returns a new array of edges.
 * This is the same as the second element of the tuple returned by React's `useState` hook.
 *
 * - `onEdgesChange`: A handy callback that can take an array of `EdgeChanges` and update the edges
 * state accordingly. You'll typically pass this directly to the `onEdgesChange` prop of your
 * `<ReactFlow />` component.
 * @example
 *
 *```tsx
 *import { ReactFlow, useNodesState, useEdgesState } from '@xyflow/react';
 *
 *const initialNodes = [];
 *const initialEdges = [];
 *
 *export default function () {
 *  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
 *  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
 *
 *  return (
 *    <ReactFlow
 *      nodes={nodes}
 *      edges={edges}
 *      onNodesChange={onNodesChange}
 *      onEdgesChange={onEdgesChange}
 *    />
 *  );
 *}
 *```
 *
 * @remarks This hook was created to make prototyping easier and our documentation
 * examples clearer. Although it is OK to use this hook in production, in
 * practice you may want to use a more sophisticated state management solution
 * like Zustand {@link https://reactflow.dev/docs/guides/state-management/} instead.
 *
 */
function useEdgesState(initialEdges) {
    const [edges, setEdges] = useState(initialEdges);
    const onEdgesChange = useCallback((changes) => setEdges((eds) => applyEdgeChanges(changes, eds)), []);
    return [edges, setEdges, onEdgesChange];
}

function LinePattern({ dimensions, lineWidth, variant, className }) {
    return (jsx("path", { strokeWidth: lineWidth, d: `M${dimensions[0] / 2} 0 V${dimensions[1]} M0 ${dimensions[1] / 2} H${dimensions[0]}`, className: cc(['react-flow__background-pattern', variant, className]) }));
}
function DotPattern({ radius, className }) {
    return (jsx("circle", { cx: radius, cy: radius, r: radius, className: cc(['react-flow__background-pattern', 'dots', className]) }));
}

/**
 * The three variants are exported as an enum for convenience. You can either import
 * the enum and use it like `BackgroundVariant.Lines` or you can use the raw string
 * value directly.
 * @public
 */
var BackgroundVariant;
(function (BackgroundVariant) {
    BackgroundVariant["Lines"] = "lines";
    BackgroundVariant["Dots"] = "dots";
    BackgroundVariant["Cross"] = "cross";
})(BackgroundVariant || (BackgroundVariant = {}));

const defaultSize = {
    [BackgroundVariant.Dots]: 1,
    [BackgroundVariant.Lines]: 1,
    [BackgroundVariant.Cross]: 6,
};
const selector$3 = (s) => ({ transform: s.transform, patternId: `pattern-${s.rfId}` });
function BackgroundComponent({ id, variant = BackgroundVariant.Dots, 
// only used for dots and cross
gap = 20, 
// only used for lines and cross
size, lineWidth = 1, offset = 0, color, bgColor, style, className, patternClassName, }) {
    const ref = useRef(null);
    const { transform, patternId } = useStore(selector$3, shallow);
    const patternSize = size || defaultSize[variant];
    const isDots = variant === BackgroundVariant.Dots;
    const isCross = variant === BackgroundVariant.Cross;
    const gapXY = Array.isArray(gap) ? gap : [gap, gap];
    const scaledGap = [gapXY[0] * transform[2] || 1, gapXY[1] * transform[2] || 1];
    const scaledSize = patternSize * transform[2];
    const offsetXY = Array.isArray(offset) ? offset : [offset, offset];
    const patternDimensions = isCross ? [scaledSize, scaledSize] : scaledGap;
    const scaledOffset = [
        offsetXY[0] * transform[2] || 1 + patternDimensions[0] / 2,
        offsetXY[1] * transform[2] || 1 + patternDimensions[1] / 2,
    ];
    const _patternId = `${patternId}${id ? id : ''}`;
    return (jsxs("svg", { className: cc(['react-flow__background', className]), style: {
            ...style,
            ...containerStyle,
            '--xy-background-color-props': bgColor,
            '--xy-background-pattern-color-props': color,
        }, ref: ref, "data-testid": "rf__background", children: [jsx("pattern", { id: _patternId, x: transform[0] % scaledGap[0], y: transform[1] % scaledGap[1], width: scaledGap[0], height: scaledGap[1], patternUnits: "userSpaceOnUse", patternTransform: `translate(-${scaledOffset[0]},-${scaledOffset[1]})`, children: isDots ? (jsx(DotPattern, { radius: scaledSize / 2, className: patternClassName })) : (jsx(LinePattern, { dimensions: patternDimensions, lineWidth: lineWidth, variant: variant, className: patternClassName })) }), jsx("rect", { x: "0", y: "0", width: "100%", height: "100%", fill: `url(#${_patternId})` })] }));
}
BackgroundComponent.displayName = 'Background';
/**
 * The `<Background />` component makes it convenient to render different types of backgrounds common in node-based UIs. It comes with three variants: lines, dots and cross.
 *
 * @example
 *
 * A simple example of how to use the Background component.
 *
 * ```tsx
 * import { useState } from 'react';
 * import { ReactFlow, Background, BackgroundVariant } from '@xyflow/react';
 *
 * export default function Flow() {
 *   return (
 *     <ReactFlow defaultNodes={[...]} defaultEdges={[...]}>
 *       <Background color="#ccc" variant={BackgroundVariant.Dots} />
 *     </ReactFlow>
 *   );
 * }
 * ```
 *
 * @example
 *
 * In this example you can see how to combine multiple backgrounds
 *
 * ```tsx
 * import { ReactFlow, Background, BackgroundVariant } from '@xyflow/react';
 * import '@xyflow/react/dist/style.css';
 *
 * export default function Flow() {
 *   return (
 *     <ReactFlow defaultNodes={[...]} defaultEdges={[...]}>
 *       <Background
 *         id="1"
 *         gap={10}
 *         color="#f1f1f1"
 *         variant={BackgroundVariant.Lines}
 *       />
 *       <Background
 *         id="2"
 *         gap={100}
 *         color="#ccc"
 *         variant={BackgroundVariant.Lines}
 *       />
 *     </ReactFlow>
 *   );
 * }
 * ```
 *
 * @remarks
 *
 * When combining multiple <Background /> components it’s important to give each of them a unique id prop!
 *
 */
const Background = memo(BackgroundComponent);

function PlusIcon() {
    return (jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 32", children: jsx("path", { d: "M32 18.133H18.133V32h-4.266V18.133H0v-4.266h13.867V0h4.266v13.867H32z" }) }));
}

function MinusIcon() {
    return (jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 5", children: jsx("path", { d: "M0 0h32v4.2H0z" }) }));
}

function FitViewIcon() {
    return (jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 30", children: jsx("path", { d: "M3.692 4.63c0-.53.4-.938.939-.938h5.215V0H4.708C2.13 0 0 2.054 0 4.63v5.216h3.692V4.631zM27.354 0h-5.2v3.692h5.17c.53 0 .984.4.984.939v5.215H32V4.631A4.624 4.624 0 0027.354 0zm.954 24.83c0 .532-.4.94-.939.94h-5.215v3.768h5.215c2.577 0 4.631-2.13 4.631-4.707v-5.139h-3.692v5.139zm-23.677.94c-.531 0-.939-.4-.939-.94v-5.138H0v5.139c0 2.577 2.13 4.707 4.708 4.707h5.138V25.77H4.631z" }) }));
}

function LockIcon() {
    return (jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 25 32", children: jsx("path", { d: "M21.333 10.667H19.81V7.619C19.81 3.429 16.38 0 12.19 0 8 0 4.571 3.429 4.571 7.619v3.048H3.048A3.056 3.056 0 000 13.714v15.238A3.056 3.056 0 003.048 32h18.285a3.056 3.056 0 003.048-3.048V13.714a3.056 3.056 0 00-3.048-3.047zM12.19 24.533a3.056 3.056 0 01-3.047-3.047 3.056 3.056 0 013.047-3.048 3.056 3.056 0 013.048 3.048 3.056 3.056 0 01-3.048 3.047zm4.724-13.866H7.467V7.619c0-2.59 2.133-4.724 4.723-4.724 2.591 0 4.724 2.133 4.724 4.724v3.048z" }) }));
}

function UnlockIcon() {
    return (jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 25 32", children: jsx("path", { d: "M21.333 10.667H19.81V7.619C19.81 3.429 16.38 0 12.19 0c-4.114 1.828-1.37 2.133.305 2.438 1.676.305 4.42 2.59 4.42 5.181v3.048H3.047A3.056 3.056 0 000 13.714v15.238A3.056 3.056 0 003.048 32h18.285a3.056 3.056 0 003.048-3.048V13.714a3.056 3.056 0 00-3.048-3.047zM12.19 24.533a3.056 3.056 0 01-3.047-3.047 3.056 3.056 0 013.047-3.048 3.056 3.056 0 013.048 3.048 3.056 3.056 0 01-3.048 3.047z" }) }));
}

/**
 * You can add buttons to the control panel by using the `<ControlButton />` component
 * and pass it as a child to the [`<Controls />`](/api-reference/components/controls) component.
 *
 * @public
 * @example
 *```jsx
 *import { MagicWand } from '@radix-ui/react-icons'
 *import { ReactFlow, Controls, ControlButton } from '@xyflow/react'
 *
 *export default function Flow() {
 *  return (
 *    <ReactFlow nodes={[...]} edges={[...]}>
 *      <Controls>
 *        <ControlButton onClick={() => alert('Something magical just happened. ✨')}>
 *          <MagicWand />
 *        </ControlButton>
 *      </Controls>
 *    </ReactFlow>
 *  )
 *}
 *```
 */
function ControlButton({ children, className, ...rest }) {
    return (jsx("button", { type: "button", className: cc(['react-flow__controls-button', className]), ...rest, children: children }));
}

const selector$2 = (s) => ({
    isInteractive: s.nodesDraggable || s.nodesConnectable || s.elementsSelectable,
    minZoomReached: s.transform[2] <= s.minZoom,
    maxZoomReached: s.transform[2] >= s.maxZoom,
    ariaLabelConfig: s.ariaLabelConfig,
});
function ControlsComponent({ style, showZoom = true, showFitView = true, showInteractive = true, fitViewOptions, onZoomIn, onZoomOut, onFitView, onInteractiveChange, className, children, position = 'bottom-left', orientation = 'vertical', 'aria-label': ariaLabel, }) {
    const store = useStoreApi();
    const { isInteractive, minZoomReached, maxZoomReached, ariaLabelConfig } = useStore(selector$2, shallow);
    const { zoomIn, zoomOut, fitView } = useReactFlow();
    const onZoomInHandler = () => {
        zoomIn();
        onZoomIn?.();
    };
    const onZoomOutHandler = () => {
        zoomOut();
        onZoomOut?.();
    };
    const onFitViewHandler = () => {
        fitView(fitViewOptions);
        onFitView?.();
    };
    const onToggleInteractivity = () => {
        store.setState({
            nodesDraggable: !isInteractive,
            nodesConnectable: !isInteractive,
            elementsSelectable: !isInteractive,
        });
        onInteractiveChange?.(!isInteractive);
    };
    const orientationClass = orientation === 'horizontal' ? 'horizontal' : 'vertical';
    return (jsxs(Panel, { className: cc(['react-flow__controls', orientationClass, className]), position: position, style: style, "data-testid": "rf__controls", "aria-label": ariaLabel ?? ariaLabelConfig['controls.ariaLabel'], children: [showZoom && (jsxs(Fragment, { children: [jsx(ControlButton, { onClick: onZoomInHandler, className: "react-flow__controls-zoomin", title: ariaLabelConfig['controls.zoomIn.ariaLabel'], "aria-label": ariaLabelConfig['controls.zoomIn.ariaLabel'], disabled: maxZoomReached, children: jsx(PlusIcon, {}) }), jsx(ControlButton, { onClick: onZoomOutHandler, className: "react-flow__controls-zoomout", title: ariaLabelConfig['controls.zoomOut.ariaLabel'], "aria-label": ariaLabelConfig['controls.zoomOut.ariaLabel'], disabled: minZoomReached, children: jsx(MinusIcon, {}) })] })), showFitView && (jsx(ControlButton, { className: "react-flow__controls-fitview", onClick: onFitViewHandler, title: ariaLabelConfig['controls.fitView.ariaLabel'], "aria-label": ariaLabelConfig['controls.fitView.ariaLabel'], children: jsx(FitViewIcon, {}) })), showInteractive && (jsx(ControlButton, { className: "react-flow__controls-interactive", onClick: onToggleInteractivity, title: ariaLabelConfig['controls.interactive.ariaLabel'], "aria-label": ariaLabelConfig['controls.interactive.ariaLabel'], children: isInteractive ? jsx(UnlockIcon, {}) : jsx(LockIcon, {}) })), children] }));
}
ControlsComponent.displayName = 'Controls';
/**
 * The `<Controls />` component renders a small panel that contains convenient
 * buttons to zoom in, zoom out, fit the view, and lock the viewport.
 *
 * @public
 * @example
 *```tsx
 *import { ReactFlow, Controls } from '@xyflow/react'
 *
 *export default function Flow() {
 *  return (
 *    <ReactFlow nodes={[...]} edges={[...]}>
 *      <Controls />
 *    </ReactFlow>
 *  )
 *}
 *```
 *
 * @remarks To extend or customise the controls, you can use the [`<ControlButton />`](/api-reference/components/control-button) component
 *
 */
const Controls = memo(ControlsComponent);

function MiniMapNodeComponent({ id, x, y, width, height, style, color, strokeColor, strokeWidth, className, borderRadius, shapeRendering, selected, onClick, }) {
    const { background, backgroundColor } = style || {};
    const fill = (color || background || backgroundColor);
    return (jsx("rect", { className: cc(['react-flow__minimap-node', { selected }, className]), x: x, y: y, rx: borderRadius, ry: borderRadius, width: width, height: height, style: {
            fill,
            stroke: strokeColor,
            strokeWidth,
        }, shapeRendering: shapeRendering, onClick: onClick ? (event) => onClick(event, id) : undefined }));
}
const MiniMapNode = memo(MiniMapNodeComponent);

const selectorNodeIds = (s) => s.nodes.map((node) => node.id);
const getAttrFunction = (func) => func instanceof Function ? func : () => func;
function MiniMapNodes({ nodeStrokeColor, nodeColor, nodeClassName = '', nodeBorderRadius = 5, nodeStrokeWidth, 
/*
 * We need to rename the prop to be `CapitalCase` so that JSX will render it as
 * a component properly.
 */
nodeComponent: NodeComponent = MiniMapNode, onClick, }) {
    const nodeIds = useStore(selectorNodeIds, shallow);
    const nodeColorFunc = getAttrFunction(nodeColor);
    const nodeStrokeColorFunc = getAttrFunction(nodeStrokeColor);
    const nodeClassNameFunc = getAttrFunction(nodeClassName);
    const shapeRendering = 'crispEdges' ;
    return (jsx(Fragment, { children: nodeIds.map((nodeId) => (
        /*
         * The split of responsibilities between MiniMapNodes and
         * NodeComponentWrapper may appear weird. However, it’s designed to
         * minimize the cost of updates when individual nodes change.
         *
         * For more details, see a similar commit in `NodeRenderer/index.tsx`.
         */
        jsx(NodeComponentWrapper, { id: nodeId, nodeColorFunc: nodeColorFunc, nodeStrokeColorFunc: nodeStrokeColorFunc, nodeClassNameFunc: nodeClassNameFunc, nodeBorderRadius: nodeBorderRadius, nodeStrokeWidth: nodeStrokeWidth, NodeComponent: NodeComponent, onClick: onClick, shapeRendering: shapeRendering }, nodeId))) }));
}
function NodeComponentWrapperInner({ id, nodeColorFunc, nodeStrokeColorFunc, nodeClassNameFunc, nodeBorderRadius, nodeStrokeWidth, shapeRendering, NodeComponent, onClick, }) {
    const { node, x, y, width, height } = useStore((s) => {
        const { internals } = s.nodeLookup.get(id);
        const node = internals.userNode;
        const { x, y } = internals.positionAbsolute;
        const { width, height } = getNodeDimensions(node);
        return {
            node,
            x,
            y,
            width,
            height,
        };
    }, shallow);
    if (!node || node.hidden || !nodeHasDimensions(node)) {
        return null;
    }
    return (jsx(NodeComponent, { x: x, y: y, width: width, height: height, style: node.style, selected: !!node.selected, className: nodeClassNameFunc(node), color: nodeColorFunc(node), borderRadius: nodeBorderRadius, strokeColor: nodeStrokeColorFunc(node), strokeWidth: nodeStrokeWidth, shapeRendering: shapeRendering, onClick: onClick, id: node.id }));
}
const NodeComponentWrapper = memo(NodeComponentWrapperInner);
var MiniMapNodes$1 = memo(MiniMapNodes);

const defaultWidth = 200;
const defaultHeight = 150;
const filterHidden = (node) => !node.hidden;
const selector$1 = (s) => {
    const viewBB = {
        x: -s.transform[0] / s.transform[2],
        y: -s.transform[1] / s.transform[2],
        width: s.width / s.transform[2],
        height: s.height / s.transform[2],
    };
    return {
        viewBB,
        boundingRect: s.nodeLookup.size > 0
            ? getBoundsOfRects(getInternalNodesBounds(s.nodeLookup, { filter: filterHidden }), viewBB)
            : viewBB,
        rfId: s.rfId,
        panZoom: s.panZoom,
        translateExtent: s.translateExtent,
        flowWidth: s.width,
        flowHeight: s.height,
        ariaLabelConfig: s.ariaLabelConfig,
    };
};
const ARIA_LABEL_KEY = 'react-flow__minimap-desc';
function MiniMapComponent({ style, className, nodeStrokeColor, nodeColor, nodeClassName = '', nodeBorderRadius = 5, nodeStrokeWidth, 
/*
 * We need to rename the prop to be `CapitalCase` so that JSX will render it as
 * a component properly.
 */
nodeComponent, bgColor, maskColor, maskStrokeColor, maskStrokeWidth, position = 'bottom-right', onClick, onNodeClick, pannable = false, zoomable = false, ariaLabel, inversePan, zoomStep = 10, offsetScale = 5, }) {
    const store = useStoreApi();
    const svg = useRef(null);
    const { boundingRect, viewBB, rfId, panZoom, translateExtent, flowWidth, flowHeight, ariaLabelConfig } = useStore(selector$1, shallow);
    const elementWidth = style?.width ?? defaultWidth;
    const elementHeight = style?.height ?? defaultHeight;
    const scaledWidth = boundingRect.width / elementWidth;
    const scaledHeight = boundingRect.height / elementHeight;
    const viewScale = Math.max(scaledWidth, scaledHeight);
    const viewWidth = viewScale * elementWidth;
    const viewHeight = viewScale * elementHeight;
    const offset = offsetScale * viewScale;
    const x = boundingRect.x - (viewWidth - boundingRect.width) / 2 - offset;
    const y = boundingRect.y - (viewHeight - boundingRect.height) / 2 - offset;
    const width = viewWidth + offset * 2;
    const height = viewHeight + offset * 2;
    const labelledBy = `${ARIA_LABEL_KEY}-${rfId}`;
    const viewScaleRef = useRef(0);
    const minimapInstance = useRef();
    viewScaleRef.current = viewScale;
    useEffect(() => {
        if (svg.current && panZoom) {
            minimapInstance.current = XYMinimap({
                domNode: svg.current,
                panZoom,
                getTransform: () => store.getState().transform,
                getViewScale: () => viewScaleRef.current,
            });
            return () => {
                minimapInstance.current?.destroy();
            };
        }
    }, [panZoom]);
    useEffect(() => {
        minimapInstance.current?.update({
            translateExtent,
            width: flowWidth,
            height: flowHeight,
            inversePan,
            pannable,
            zoomStep,
            zoomable,
        });
    }, [pannable, zoomable, inversePan, zoomStep, translateExtent, flowWidth, flowHeight]);
    const onSvgClick = onClick
        ? (event) => {
            const [x, y] = minimapInstance.current?.pointer(event) || [0, 0];
            onClick(event, { x, y });
        }
        : undefined;
    const onSvgNodeClick = onNodeClick
        ? useCallback((event, nodeId) => {
            const node = store.getState().nodeLookup.get(nodeId).internals.userNode;
            onNodeClick(event, node);
        }, [])
        : undefined;
    const _ariaLabel = ariaLabel ?? ariaLabelConfig['minimap.ariaLabel'];
    return (jsx(Panel, { position: position, style: {
            ...style,
            '--xy-minimap-background-color-props': typeof bgColor === 'string' ? bgColor : undefined,
            '--xy-minimap-mask-background-color-props': typeof maskColor === 'string' ? maskColor : undefined,
            '--xy-minimap-mask-stroke-color-props': typeof maskStrokeColor === 'string' ? maskStrokeColor : undefined,
            '--xy-minimap-mask-stroke-width-props': typeof maskStrokeWidth === 'number' ? maskStrokeWidth * viewScale : undefined,
            '--xy-minimap-node-background-color-props': typeof nodeColor === 'string' ? nodeColor : undefined,
            '--xy-minimap-node-stroke-color-props': typeof nodeStrokeColor === 'string' ? nodeStrokeColor : undefined,
            '--xy-minimap-node-stroke-width-props': typeof nodeStrokeWidth === 'number' ? nodeStrokeWidth : undefined,
        }, className: cc(['react-flow__minimap', className]), "data-testid": "rf__minimap", children: jsxs("svg", { width: elementWidth, height: elementHeight, viewBox: `${x} ${y} ${width} ${height}`, className: "react-flow__minimap-svg", role: "img", "aria-labelledby": labelledBy, ref: svg, onClick: onSvgClick, children: [_ariaLabel && jsx("title", { id: labelledBy, children: _ariaLabel }), jsx(MiniMapNodes$1, { onClick: onSvgNodeClick, nodeColor: nodeColor, nodeStrokeColor: nodeStrokeColor, nodeBorderRadius: nodeBorderRadius, nodeClassName: nodeClassName, nodeStrokeWidth: nodeStrokeWidth, nodeComponent: nodeComponent }), jsx("path", { className: "react-flow__minimap-mask", d: `M${x - offset},${y - offset}h${width + offset * 2}v${height + offset * 2}h${-width - offset * 2}z
        M${viewBB.x},${viewBB.y}h${viewBB.width}v${viewBB.height}h${-viewBB.width}z`, fillRule: "evenodd", pointerEvents: "none" })] }) }));
}
MiniMapComponent.displayName = 'MiniMap';
/**
 * The `<MiniMap />` component can be used to render an overview of your flow. It
 * renders each node as an SVG element and visualizes where the current viewport is
 * in relation to the rest of the flow.
 *
 * @public
 * @example
 *
 * ```jsx
 *import { ReactFlow, MiniMap } from '@xyflow/react';
 *
 *export default function Flow() {
 *  return (
 *    <ReactFlow nodes={[...]]} edges={[...]]}>
 *      <MiniMap nodeStrokeWidth={3} />
 *    </ReactFlow>
 *  );
 *}
 *```
 */
memo(MiniMapComponent);

const scaleSelector = (calculateScale) => (store) => calculateScale ? `${Math.max(1 / store.transform[2], 1)}` : undefined;
const defaultPositions = {
    [ResizeControlVariant.Line]: 'right',
    [ResizeControlVariant.Handle]: 'bottom-right',
};
function ResizeControl({ nodeId, position, variant = ResizeControlVariant.Handle, className, style = undefined, children, color, minWidth = 10, minHeight = 10, maxWidth = Number.MAX_VALUE, maxHeight = Number.MAX_VALUE, keepAspectRatio = false, resizeDirection, autoScale = true, shouldResize, onResizeStart, onResize, onResizeEnd, }) {
    const contextNodeId = useNodeId();
    const id = typeof nodeId === 'string' ? nodeId : contextNodeId;
    const store = useStoreApi();
    const resizeControlRef = useRef(null);
    const isHandleControl = variant === ResizeControlVariant.Handle;
    const scale = useStore(useCallback(scaleSelector(isHandleControl && autoScale), [isHandleControl, autoScale]), shallow);
    const resizer = useRef(null);
    const controlPosition = position ?? defaultPositions[variant];
    useEffect(() => {
        if (!resizeControlRef.current || !id) {
            return;
        }
        if (!resizer.current) {
            resizer.current = XYResizer({
                domNode: resizeControlRef.current,
                nodeId: id,
                getStoreItems: () => {
                    const { nodeLookup, transform, snapGrid, snapToGrid, nodeOrigin, domNode } = store.getState();
                    return {
                        nodeLookup,
                        transform,
                        snapGrid,
                        snapToGrid,
                        nodeOrigin,
                        paneDomNode: domNode,
                    };
                },
                onChange: (change, childChanges) => {
                    const { triggerNodeChanges, nodeLookup, parentLookup, nodeOrigin } = store.getState();
                    const changes = [];
                    const nextPosition = { x: change.x, y: change.y };
                    const node = nodeLookup.get(id);
                    if (node && node.expandParent && node.parentId) {
                        const origin = node.origin ?? nodeOrigin;
                        const width = change.width ?? node.measured.width ?? 0;
                        const height = change.height ?? node.measured.height ?? 0;
                        const child = {
                            id: node.id,
                            parentId: node.parentId,
                            rect: {
                                width,
                                height,
                                ...evaluateAbsolutePosition({
                                    x: change.x ?? node.position.x,
                                    y: change.y ?? node.position.y,
                                }, { width, height }, node.parentId, nodeLookup, origin),
                            },
                        };
                        const parentExpandChanges = handleExpandParent([child], nodeLookup, parentLookup, nodeOrigin);
                        changes.push(...parentExpandChanges);
                        /*
                         * when the parent was expanded by the child node, its position will be clamped at
                         * 0,0 when node origin is 0,0 and to width, height if it's 1,1
                         */
                        nextPosition.x = change.x ? Math.max(origin[0] * width, change.x) : undefined;
                        nextPosition.y = change.y ? Math.max(origin[1] * height, change.y) : undefined;
                    }
                    if (nextPosition.x !== undefined && nextPosition.y !== undefined) {
                        const positionChange = {
                            id,
                            type: 'position',
                            position: { ...nextPosition },
                        };
                        changes.push(positionChange);
                    }
                    if (change.width !== undefined && change.height !== undefined) {
                        const setAttributes = !resizeDirection ? true : resizeDirection === 'horizontal' ? 'width' : 'height';
                        const dimensionChange = {
                            id,
                            type: 'dimensions',
                            resizing: true,
                            setAttributes,
                            dimensions: {
                                width: change.width,
                                height: change.height,
                            },
                        };
                        changes.push(dimensionChange);
                    }
                    for (const childChange of childChanges) {
                        const positionChange = {
                            ...childChange,
                            type: 'position',
                        };
                        changes.push(positionChange);
                    }
                    triggerNodeChanges(changes);
                },
                onEnd: ({ width, height }) => {
                    const dimensionChange = {
                        id: id,
                        type: 'dimensions',
                        resizing: false,
                        dimensions: {
                            width,
                            height,
                        },
                    };
                    store.getState().triggerNodeChanges([dimensionChange]);
                },
            });
        }
        resizer.current.update({
            controlPosition,
            boundaries: {
                minWidth,
                minHeight,
                maxWidth,
                maxHeight,
            },
            keepAspectRatio,
            resizeDirection,
            onResizeStart,
            onResize,
            onResizeEnd,
            shouldResize,
        });
        return () => {
            resizer.current?.destroy();
        };
    }, [
        controlPosition,
        minWidth,
        minHeight,
        maxWidth,
        maxHeight,
        keepAspectRatio,
        onResizeStart,
        onResize,
        onResizeEnd,
        shouldResize,
    ]);
    const positionClassNames = controlPosition.split('-');
    return (jsx("div", { className: cc(['react-flow__resize-control', 'nodrag', ...positionClassNames, variant, className]), ref: resizeControlRef, style: {
            ...style,
            scale,
            ...(color && { [isHandleControl ? 'backgroundColor' : 'borderColor']: color }),
        }, children: children }));
}
/**
 * To create your own resizing UI, you can use the `NodeResizeControl` component where you can pass children (such as icons).
 * @public
 *
 */
memo(ResizeControl);

const API_BASE_URL = "http://localhost:3000/api";
const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    "Content-Type": "application/json"
  }
});
const threadsApi = {
  listThreads: async () => {
    const response = await api.get("/threads");
    return response.data.threads;
  },
  getThread: async (id) => {
    const response = await api.get(`/threads/${id}`);
    return response.data;
  },
  createThread: async (goal) => {
    const response = await api.post("/threads", { goal });
    return response.data;
  }
};
const statusColors = {
  pending: "#9CA3AF",
  // gray
  running: "#F59E0B",
  // yellow
  waiting: "#3B82F6",
  // blue
  completed: "#10B981",
  // green
  failed: "#EF4444"
  // red
};
const ThreadNode = ({ data }) => {
  const { thread, onClick } = data;
  const color = statusColors[thread.status];
  return /* @__PURE__ */ jsxs(
    "div",
    {
      style: {
        background: "white",
        border: `2px solid ${color}`,
        borderRadius: "8px",
        padding: "10px",
        minWidth: "200px",
        cursor: "pointer"
      },
      onClick: () => onClick(thread),
      children: [
        /* @__PURE__ */ jsx(Handle, { type: "target", position: Position.Top }),
        /* @__PURE__ */ jsx("div", { style: { fontWeight: "bold", marginBottom: "4px" }, children: "Thread" }),
        /* @__PURE__ */ jsxs("div", { style: { fontSize: "12px", color: "#666" }, children: [
          thread.goal.substring(0, 50),
          "..."
        ] }),
        /* @__PURE__ */ jsx("div", { style: { fontSize: "10px", color, marginTop: "4px" }, children: thread.status.toUpperCase() }),
        thread.tasks.length > 0 && /* @__PURE__ */ jsxs("div", { style: { fontSize: "10px", marginTop: "4px" }, children: [
          "Tasks: ",
          thread.tasks.filter((t) => t.status === "completed").length,
          "/",
          thread.tasks.length
        ] }),
        /* @__PURE__ */ jsx(Handle, { type: "source", position: Position.Bottom })
      ]
    }
  );
};
const stitchIcons = {
  llm_call: "\u{1F916}",
  tool_call: "\u{1F527}",
  thread_result: "\u{1F4CA}"
};
const StitchNode = ({ data }) => {
  const { stitch, onClick } = data;
  const icon = stitchIcons[stitch.stitch_type];
  return /* @__PURE__ */ jsxs(
    "div",
    {
      style: {
        background: "#f0f0f0",
        border: "1px solid #ccc",
        borderRadius: "6px",
        padding: "8px",
        minWidth: "150px",
        cursor: "pointer",
        fontSize: "12px"
      },
      onClick: () => onClick(stitch),
      children: [
        /* @__PURE__ */ jsx(Handle, { type: "target", position: Position.Top }),
        /* @__PURE__ */ jsxs("div", { style: { display: "flex", alignItems: "center", gap: "4px" }, children: [
          /* @__PURE__ */ jsx("span", { children: icon }),
          /* @__PURE__ */ jsx("span", { children: stitch.stitch_type.replace("_", " ") })
        ] }),
        stitch.tool_name && /* @__PURE__ */ jsx("div", { style: { fontSize: "10px", color: "#666", marginTop: "2px" }, children: stitch.tool_name }),
        stitch.thread_result_summary && /* @__PURE__ */ jsxs("div", { style: { fontSize: "10px", color: "#666", marginTop: "2px" }, children: [
          stitch.thread_result_summary.substring(0, 50),
          "..."
        ] }),
        /* @__PURE__ */ jsx(Handle, { type: "source", position: Position.Bottom })
      ]
    }
  );
};
const ThreadDetailPanel = ({ thread, stitch, onClose }) => {
  if (!thread && !stitch) return null;
  return /* @__PURE__ */ jsxs(
    "div",
    {
      style: {
        position: "absolute",
        right: 0,
        top: 0,
        bottom: 0,
        width: "400px",
        background: "white",
        borderLeft: "1px solid #ccc",
        padding: "20px",
        overflowY: "auto",
        zIndex: 10
      },
      children: [
        /* @__PURE__ */ jsx(
          "button",
          {
            onClick: onClose,
            style: {
              position: "absolute",
              right: "10px",
              top: "10px",
              background: "none",
              border: "none",
              fontSize: "20px",
              cursor: "pointer"
            },
            children: "\xD7"
          }
        ),
        thread && /* @__PURE__ */ jsxs("div", { children: [
          /* @__PURE__ */ jsx("h2", { style: { marginTop: 0 }, children: "Thread Details" }),
          /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "ID:" }),
            " ",
            thread.id
          ] }),
          /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Goal:" }),
            " ",
            thread.goal
          ] }),
          /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Status:" }),
            " ",
            thread.status
          ] }),
          thread.tasks.length > 0 && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Tasks:" }),
            /* @__PURE__ */ jsx("ul", { style: { marginTop: "5px" }, children: thread.tasks.map((task, index) => /* @__PURE__ */ jsxs("li", { style: { marginBottom: "5px" }, children: [
              /* @__PURE__ */ jsxs("span", { style: {
                color: task.status === "completed" ? "green" : task.status === "in_progress" ? "orange" : "gray"
              }, children: [
                "[",
                task.status,
                "]"
              ] }),
              " ",
              task.description
            ] }, index)) })
          ] }),
          thread.result && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Result:" }),
            /* @__PURE__ */ jsx("pre", { style: {
              background: "#f5f5f5",
              padding: "10px",
              borderRadius: "4px",
              fontSize: "12px",
              overflow: "auto"
            }, children: JSON.stringify(thread.result, null, 2) })
          ] })
        ] }),
        stitch && /* @__PURE__ */ jsxs("div", { children: [
          /* @__PURE__ */ jsx("h2", { style: { marginTop: 0 }, children: "Stitch Details" }),
          /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "ID:" }),
            " ",
            stitch.id
          ] }),
          /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Type:" }),
            " ",
            stitch.stitch_type
          ] }),
          stitch.tool_name && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Tool:" }),
            " ",
            stitch.tool_name
          ] }),
          stitch.llm_request && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "LLM Request:" }),
            /* @__PURE__ */ jsx("pre", { style: {
              background: "#f5f5f5",
              padding: "10px",
              borderRadius: "4px",
              fontSize: "12px",
              overflow: "auto"
            }, children: JSON.stringify(stitch.llm_request, null, 2) })
          ] }),
          stitch.llm_response && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "LLM Response:" }),
            /* @__PURE__ */ jsx("pre", { style: {
              background: "#f5f5f5",
              padding: "10px",
              borderRadius: "4px",
              fontSize: "12px",
              overflow: "auto"
            }, children: JSON.stringify(stitch.llm_response, null, 2) })
          ] }),
          stitch.tool_input && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Tool Input:" }),
            /* @__PURE__ */ jsx("pre", { style: {
              background: "#f5f5f5",
              padding: "10px",
              borderRadius: "4px",
              fontSize: "12px",
              overflow: "auto"
            }, children: JSON.stringify(stitch.tool_input, null, 2) })
          ] }),
          stitch.tool_output && /* @__PURE__ */ jsxs("div", { style: { marginBottom: "10px" }, children: [
            /* @__PURE__ */ jsx("strong", { children: "Tool Output:" }),
            /* @__PURE__ */ jsx("pre", { style: {
              background: "#f5f5f5",
              padding: "10px",
              borderRadius: "4px",
              fontSize: "12px",
              overflow: "auto"
            }, children: JSON.stringify(stitch.tool_output, null, 2) })
          ] })
        ] })
      ]
    }
  );
};
const nodeTypes = {
  thread: ThreadNode,
  stitch: StitchNode
};
const ThreadGraphView = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [selectedThread, setSelectedThread] = useState();
  const [selectedStitch, setSelectedStitch] = useState();
  const [isLoading, setIsLoading] = useState(true);
  const fetchThreads = useCallback(async () => {
    try {
      const threads = await threadsApi.listThreads();
      const newNodes = [];
      const newEdges = [];
      threads.forEach((thread, index) => {
        newNodes.push({
          id: `thread-${thread.id}`,
          type: "thread",
          position: { x: index * 300, y: 0 },
          data: {
            thread,
            onClick: async (t) => {
              setSelectedThread(t);
              setSelectedStitch(void 0);
              const fullThread = await threadsApi.getThread(t.id);
              setSelectedThread(fullThread);
            }
          }
        });
        if (thread.parent_thread_id) {
          newEdges.push({
            id: `edge-${thread.parent_thread_id}-${thread.id}`,
            source: `thread-${thread.parent_thread_id}`,
            target: `thread-${thread.id}`
          });
        }
      });
      setNodes(newNodes);
      setEdges(newEdges);
      setIsLoading(false);
    } catch (error) {
      console.error("Failed to fetch threads:", error);
      setIsLoading(false);
    }
  }, [setNodes, setEdges]);
  useEffect(() => {
    fetchThreads();
    const interval = setInterval(fetchThreads, 2e3);
    return () => clearInterval(interval);
  }, [fetchThreads]);
  const handlePaneClick = useCallback(() => {
    setSelectedThread(void 0);
    setSelectedStitch(void 0);
  }, []);
  if (isLoading) {
    return /* @__PURE__ */ jsx("div", { style: {
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      height: "100vh"
    }, children: "Loading threads..." });
  }
  return /* @__PURE__ */ jsxs("div", { style: { width: "100vw", height: "100vh", position: "relative" }, children: [
    /* @__PURE__ */ jsxs(
      index,
      {
        nodes,
        edges,
        onNodesChange,
        onEdgesChange,
        onPaneClick: handlePaneClick,
        nodeTypes,
        fitView: true,
        children: [
          /* @__PURE__ */ jsx(Controls, {}),
          /* @__PURE__ */ jsx(Background, { variant: BackgroundVariant.Dots, gap: 12, size: 1 })
        ]
      }
    ),
    /* @__PURE__ */ jsx(
      ThreadDetailPanel,
      {
        thread: selectedThread,
        stitch: selectedStitch,
        onClose: () => {
          setSelectedThread(void 0);
          setSelectedStitch(void 0);
        }
      }
    ),
    /* @__PURE__ */ jsxs("div", { style: {
      position: "absolute",
      top: "10px",
      left: "10px",
      background: "white",
      padding: "10px",
      borderRadius: "8px",
      boxShadow: "0 2px 4px rgba(0,0,0,0.1)"
    }, children: [
      /* @__PURE__ */ jsx("h3", { style: { margin: "0 0 10px 0" }, children: "Agentic Threads Visualization" }),
      /* @__PURE__ */ jsxs("div", { style: { fontSize: "12px", color: "#666" }, children: [
        /* @__PURE__ */ jsx("div", { children: "\u2022 Click a thread to see details" }),
        /* @__PURE__ */ jsx("div", { children: "\u2022 Auto-refreshes every 2 seconds" })
      ] })
    ] })
  ] });
};
const SplitComponent = function Home() {
  return /* @__PURE__ */ jsx(ThreadGraphView, {});
};

export { SplitComponent as component };
//# sourceMappingURL=index-Y3Am5uAI.mjs.map
