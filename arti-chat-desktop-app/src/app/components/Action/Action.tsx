import React from "react";
import { ActionType } from "./ActionType";
import "./Action.scss";

export default function Action({label, description, actionType, onClick, success, checked, status}) {
   
    const successIcon = (success) => {
        if (success === false) {
            return (
                <img
                    className="form__submit-btn__status"
                    alt="Failed"
                    src="/src/assets/button-failed.png"
                />
            );
        }

        return (
            <img
                className="form__submit-btn__status"
                alt="Success"
                src="/src/assets/button-success.png"
            />
        );
    }

    const renderButton = (actionType, onClick) => {
        switch (actionType) {
            case ActionType.Delete:
                var buttonImage = (success) => {
                    if (success === null) {
                        return (
                            <img
                                className="action__button__image"
                                src="/src/assets/delete.png"
                            />
                        )
                    }

                    return successIcon(success);
                };

                return (
                    <button
                        className="action__button action__button--delete"
                        onClick={onClick}
                    >
                        { buttonImage(success) }
                        Delete
                    </button>
                )

            case ActionType.Reset:
                var buttonImage = (success) => {
                    if (success === null) {
                        return (
                            <img
                                className="action__button__image"
                                src="/src/assets/reset.png"
                            />
                        )
                    }

                    return successIcon(success);
                };

                return (
                    <button
                        className="action__button action__button--reset"
                        onClick={onClick}
                    >
                        { buttonImage(success) }
                        Reset
                    </button>
                )

            case ActionType.Toggle:
                return (
                    <label className="action__toggle">
                        <input
                            type="checkbox"
                            checked={checked}
                            onChange={(e) => onClick(e.target.checked)}
                        />
                        <span className="action__toggle__slider" />
                    </label>
                );

            case ActionType.Status:
                return (
                    <>
                        {status === null && <span className="action__status">Checking...</span>}
                        {status === true && <span className="action__status">{successIcon(status)} Connected</span>}
                        {status === false && <span className="action__status">{successIcon(status)} Offline</span>}
                    </>
                )
        }
    };

    return (
        <div className="action">
            <span className="action__label">{label}</span>
            <div className="action__content">
                <span className="action__description">{description}</span>
                { renderButton(actionType, onClick) }
            </div>
        </div>
    );
}

